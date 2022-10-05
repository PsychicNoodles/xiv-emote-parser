use strum_macros::Display;

#[derive(Debug, PartialEq, Eq, Clone, Display)]
pub enum Gender {
    Male,
    Female,
}
