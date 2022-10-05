//! Log messages access static global data that holds character data.

use thiserror::Error;

use super::types::Gender;

#[derive(Debug, Clone, Error)]
#[error("Invalid parameters ({params:?}) for prop ({name:?})")]
pub struct LogMessagePropError {
    // todo make this an enum
    name: &'static str,
    params: Vec<String>,
}

type Result<T> = std::result::Result<T, LogMessagePropError>;

pub struct LogMessageProps {
    object_parameter: ObjectProp,
    player_parameter: PlayerProp,
}

/// involved characters
pub struct ObjectProp {
    me: String,
    origin: String,
    target: Option<String>,
}

impl ObjectProp {
    /// generally used just to identify who the origin and target are and differentiate
    /// but will be used as is sometimes when origin is not self or a player
    ///
    /// me: current player character
    /// origin: origin of the message
    /// target: target of the message
    pub fn new<M, O, T>(me: M, origin: O, target: Option<T>) -> ObjectProp
    where
        M: ToString,
        O: ToString,
        T: ToString,
    {
        ObjectProp {
            me: me.to_string(),
            origin: origin.to_string(),
            target: target.map(|s| s.to_string()),
        }
    }

    fn at(&self, ind: u32) -> Result<&String> {
        match ind {
            1 => Ok(&self.origin),
            2 => Ok(&self.me),
            3 => self.target.as_ref().ok_or(LogMessagePropError {
                name: "ObjectParameter",
                params: vec![ind.to_string()],
            }),
            _ => Err(LogMessagePropError {
                name: "ObjectParameter",
                params: vec![ind.to_string()],
            }),
        }
    }
}

/// seems like a grab-bag of player related attributes?
pub struct PlayerProp {
    origin: Option<Player>,
    target: Option<Player>,
    player_gender: Gender,
}

/// used by Sheets as a reference to a particular player
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    name: String,
    world: String,
    /// only used by target currently (in en and ja, second-person pronoun is ungendered)
    gender: Gender,
}

impl ToString for Player {
    fn to_string(&self) -> String {
        format!("{}@{} ({})", self.name, self.world, self.gender.to_string())
    }
}

impl Player {
    pub fn new<N, W>(name: N, world: W, gender: Gender) -> Player
    where
        N: ToString,
        W: ToString,
    {
        Player {
            name: name.to_string(),
            world: world.to_string(),
            gender,
        }
    }
}

/// PlayerParameter returns different types depending on its param
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PParam<'a> {
    /// 7, 8
    Player(Option<&'a Player>),
    /// 5
    Bool(bool),
}

impl PlayerProp {
    /// there seem to be a lot of fields in this, so might need to change to
    /// parameter struct at some point
    ///
    /// origin_player: if the origin is a player, info on that player
    /// target_player: if the target is a player, info on that player
    pub fn new(
        origin: Option<Player>,
        target: Option<Player>,
        player_gender: Gender,
    ) -> PlayerProp {
        PlayerProp {
            origin,
            target,
            player_gender,
        }
    }

    fn at(&self, ind: u32) -> Result<PParam> {
        match ind {
            // origin player info
            7 => Ok(PParam::Player(self.origin.as_ref())),
            // target player info (if targetted)
            8 => Ok(PParam::Player(self.target.as_ref())),
            // is origin player female?
            // in en at least, second-person pronouns are ungendered (yourself), and
            // PlayerParameter(5) is seemingly only used when not referring to the player
            // and if origin is not a player as a sort of default (assuming that
            // PlayerParameter(5) is not where target NPC gender info is stored)
            5 => Ok(PParam::Bool(self.player_gender == Gender::Female)),
            _ => Err(LogMessagePropError {
                name: "PlayerParameter",
                params: vec![ind.to_string()],
            }),
        }
    }
}

/// SheetEn seems to always take ObjStr as first param (unlike Sheet) and is only
/// used to get the full origin or target player names
///
/// it's not clear what all the params represent, but there's also a fixed set of
/// param combinations used, so for now just expect those
fn sheet_en(p1: u32, player: &Player, p3: u32, p4: u32) -> Result<String> {
    match (p1, p3, p4) {
        // only usage - get player name with world (if on other world)
        (2, 1, 1) => Ok(format!("{}@{}", player.name, player.world)),
        _ => Err(LogMessagePropError {
            name: "SheetEn",
            params: vec![
                p1.to_string(),
                player.to_string(),
                p3.to_string(),
                p4.to_string(),
            ],
        }),
    }
}

// pub enum SheetType {
//     ObjStr,
//     Attributive,
//     BNpcName,
// }

// pub enum SheetParam<'a> {
//     Number(u32),
//     PParam(PParam<'a>),
// }

impl LogMessageProps {
    pub fn new(object_parameter: ObjectProp, player_parameter: PlayerProp) -> LogMessageProps {
        LogMessageProps {
            object_parameter,
            player_parameter,
        }
    }

    pub fn object_parameter(&self, ind: u32) -> Result<&String> {
        self.object_parameter.at(ind)
    }

    pub fn player_parameter(&self, ind: u32) -> Result<PParam> {
        self.player_parameter.at(ind)
    }

    pub fn sheet_en(p1: u32, player: &Player, p3: u32, p4: u32) -> Result<String> {
        sheet_en(p1, player, p3, p4)
    }

    /// it's not clear what all the params represent, but there's also a fixed set of
    /// param combinations used, so for now just expect those
    pub fn sheet_objstr(player: &Player, p2: u32) -> Result<String> {
        match (player, p2) {
            // only usage - get player name with world (if on other worl)
            (player, 0) => Ok(format!("{}@{}", player.name, player.world)),
            _ => Err(LogMessagePropError {
                name: "Sheet",
                params: vec!["ObjStr".to_string(), player.to_string(), p2.to_string()],
            }),
        }
    }

    pub fn sheet_bnpcname(player: &Player, p2: u32) -> Result<bool> {
        // despite the name, seems to be used for player characters as well
        // also appears to only be used inside If tags, despite Sheet also being a tag
        match (player, p2) {
            (player, 6) => Ok(player.gender == Gender::Female),
            _ => Err(LogMessagePropError {
                name: "Sheet",
                params: vec!["BNpcName".to_string(), player.to_string(), p2.to_string()],
            }),
        }
    }
}
