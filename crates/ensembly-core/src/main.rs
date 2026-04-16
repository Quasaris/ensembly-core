mod db;

use db::{DbManager, ItemRow};
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Resolve data directory:
    //   1. ENSEMBLY_DATA_DIR env var (always wins)
    //   2. Debug builds → workspace target/dev-data/ (keeps dev off real app data)
    //   3. Release builds → platform app data dir
    let data_dir = if let Ok(override_dir) = std::env::var("ENSEMBLY_DATA_DIR") {
        std::path::PathBuf::from(override_dir)
    } else if cfg!(debug_assertions) {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/dev-data")
    } else {
        dirs::data_dir()
            .expect("could not determine data directory")
            .join("Ensembly")
    };

    // Create directory structure on first run
    let collections_dir = data_dir.join("collections");
    fs::create_dir_all(&collections_dir)?;

    let db_path = data_dir.join("ensembly.db");
    println!("Data directory: {}", data_dir.display());

    // Initialise DB and run bootstrap migration
    let db: DbManager = DbManager::open(db_path.to_str().unwrap()).await?;
    db.migrate().await?;
    println!("Database ready at {}", db_path.display());

    // Smoke-test: insert a hardcoded PoC item and read it back
    let poc_item = ItemRow {
        id: "poc-item-001".into(),
        collection_id: "books".into(),
        title: "The Name of the Wind".into(),
        tags: vec!["fantasy".into(), "fiction".into()],
        file_path: collections_dir.join("poc-item-001.json").to_string_lossy().into(),
        last_modified: 0,
    };

    db.upsert_item(&poc_item).await?;
    println!("Inserted PoC item: {}", poc_item.id);

    let fetched = db.get_item("poc-item-001").await?;
    match fetched {
        Some(item) => {
            assert_eq!(item.id, poc_item.id);
            assert_eq!(item.title, poc_item.title);
            assert_eq!(item.tags, poc_item.tags);
            println!("Smoke-test passed: read back '{}'", item.title);
        }
        None => panic!("Smoke-test failed: item not found after insert"),
    }

    Ok(())
}
