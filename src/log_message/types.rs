#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Gender {
    Male,
    Female,
}

impl ToString for Gender {
    fn to_string(&self) -> String {
        match self {
            Gender::Male => "M".to_string(),
            Gender::Female => "F".to_string(),
        }
    }
}
