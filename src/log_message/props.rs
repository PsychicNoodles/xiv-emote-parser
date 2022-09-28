//! Log messages access static global data that holds character data.

use thiserror::Error;

use super::types::Gender;

#[derive(Debug, Error)]
#[error("Invalid parameters for log message var")]
pub struct LogMessageVarError;

type Result<T> = std::result::Result<T, LogMessageVarError>;

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
    pub fn new(me: String, origin: String, target: Option<String>) -> ObjectProp {
        ObjectProp { me, origin, target }
    }

    fn at(&self, ind: u32) -> Result<&String> {
        match ind {
            1 => Ok(&self.origin),
            2 => Ok(&self.me),
            2 => self.target.as_ref().ok_or(LogMessageVarError),
            _ => Err(LogMessageVarError),
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
#[derive(Debug, Clone)]
pub struct Player {
    name: String,
    world: String,
    /// only used by target currently (in en and ja, second-person pronoun is ungendered)
    gender: Gender,
}

impl Player {
    pub fn new(name: String, world: String, gender: Gender) -> Player {
        Player {
            name,
            world,
            gender,
        }
    }
}

/// PlayerParameter returns different types depending on its param
pub enum PParam {
    /// 7, 8
    Player(Option<Player>),
    /// 5
    IsFemale(bool),
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
            7 => Ok(PParam::Player(self.origin.clone())),
            // target player info (if targetted)
            8 => Ok(PParam::Player(self.target.clone())),
            // is origin player female?
            // in en at least, second-person pronouns are ungendered (yourself), and
            // PlayerParameter(5) is seemingly only used when not referring to the player
            // and if origin is not a player as a sort of default (assuming that
            // PlayerParameter(5) is not where target NPC gender info is stored)
            5 => Ok(PParam::IsFemale(self.player_gender == Gender::Female)),
            _ => Err(LogMessageVarError),
        }
    }
}

/// SheetEn seems to always take ObjStr as first param (unlike Sheet) and is only
/// used to get the full origin or target player names
///
/// it's not clear what all the params represent, but there's also a fixed set of
/// param combinations used, so for now just expect those
fn sheet_en(p1: u32, player: Player, p3: u32, p4: u32) -> Result<String> {
    match (p1, p3, p4) {
        // only usage - get player name with world (if on other world)
        (2, 1, 1) => Ok(format!("{}@{}", player.name, player.world)),
        _ => Err(LogMessageVarError),
    }
}

// pub enum SheetType {
//     ObjStr,
//     Attributive,
//     BNpcName,
// }

pub enum SheetParam {
    Number(u32),
    PParam(PParam),
}

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

    /// it's not clear what all the params represent, but there's also a fixed set of
    /// param combinations used, so for now just expect those
    pub fn sheet_objstr(p1: SheetParam, p2: u32) -> Result<String> {
        match (p1, p2) {
            // only usage - get player name with world (if on other world)
            (SheetParam::PParam(PParam::Player(Some(player))), 0) => {
                Ok(format!("{}@{}", player.name, player.world))
            }
            _ => Err(LogMessageVarError),
        }
    }

    pub fn sheet_bnpcname(p1: SheetParam, p2: u32) -> Result<bool> {
        // despite the name, seems to be used for player characters as well
        // also appears to only be used inside If tags, despite Sheet also being a tag
        match (p1, p2) {
            (SheetParam::PParam(PParam::Player(Some(player))), 6) => {
                Ok(player.gender == Gender::Female)
            }
            _ => Err(LogMessageVarError),
        }
    }
}
