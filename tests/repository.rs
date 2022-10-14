use xiv_emote_parser::repository::{LogMessageRepository, LogMessageRepositoryError};

#[test]
fn can_load_from_xivapi() -> Result<(), LogMessageRepositoryError> {
    let repo = LogMessageRepository::from_xivapi_blocking(None)?;
    assert!(repo.emote_list().count() > 0);
    Ok(())
}
