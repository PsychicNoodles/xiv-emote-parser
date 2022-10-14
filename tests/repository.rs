use xiv_emote_parser::repository::{LogMessageRepository, LogMessageRepositoryError};

#[test]
fn can_load_from_xivapi() -> Result<(), LogMessageRepositoryError> {
    LogMessageRepository::from_xivapi_blocking(None)?;
    Ok(())
}
