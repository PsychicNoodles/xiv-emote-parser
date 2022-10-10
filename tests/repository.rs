use xiv_emote_parser::repository::{LogMessageRepository, LogMessageRepositoryError};

#[test]
fn can_load_from_xivapi() -> Result<(), LogMessageRepositoryError> {
    let repo = LogMessageRepository::from_xivapi_blocking(None)?;

    assert!(repo.all_messages().is_ok());

    Ok(())
}
