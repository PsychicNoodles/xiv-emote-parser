use xiv_emote_parser::repository::{LogMessageRepository, LogMessageRepositoryError};

#[test]
fn can_load_from_xivapi() -> Result<(), LogMessageRepositoryError> {
    pretty_env_logger::init();
    let repo = LogMessageRepository::from_xivapi(None)?;
    println!("loaded {} emotes", repo.emote_list().count());
    assert!(repo.emote_list().count() > 0);
    Ok(())
}
