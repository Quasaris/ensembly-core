use libsql::{Builder, Connection, Database};

pub struct DbManager {
    db: Database,
}

#[derive(Debug)]
pub struct ItemRow {
    pub id: String,
    pub collection_id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub file_path: String,
    pub last_modified: i64,
}

impl DbManager {
    pub async fn open(db_path: &str) -> anyhow::Result<Self> {
        let db = Builder::new_local(db_path).build().await?;
        Ok(Self { db })
    }

    fn conn(&self) -> anyhow::Result<Connection> {
        Ok(self.db.connect()?)
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        self.conn()?.execute_batch(
            "CREATE TABLE IF NOT EXISTS items (
                id            TEXT PRIMARY KEY,
                collection_id TEXT NOT NULL,
                title         TEXT NOT NULL,
                tags          TEXT,
                file_path     TEXT NOT NULL,
                last_modified INTEGER NOT NULL
            );",
        ).await?;
        Ok(())
    }

    pub async fn upsert_item(&self, row: &ItemRow) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&row.tags)?;
        self.conn()?.execute(
            "INSERT INTO items (id, collection_id, title, tags, file_path, last_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
                collection_id = excluded.collection_id,
                title         = excluded.title,
                tags          = excluded.tags,
                file_path     = excluded.file_path,
                last_modified = excluded.last_modified",
            libsql::params![
                row.id.as_str(),
                row.collection_id.as_str(),
                row.title.as_str(),
                tags_json.as_str(),
                row.file_path.as_str(),
                row.last_modified
            ],
        ).await?;
        Ok(())
    }

    pub async fn get_item(&self, id: &str) -> anyhow::Result<Option<ItemRow>> {
        let conn = self.conn()?;
        let mut rows: libsql::Rows = conn.query(
            "SELECT id, collection_id, title, tags, file_path, last_modified
             FROM items WHERE id = ?1",
            libsql::params![id],
        ).await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(row_to_item(&row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn query_items(&self, collection_id: &str) -> anyhow::Result<Vec<ItemRow>> {
        let conn = self.conn()?;
        let mut rows: libsql::Rows = conn.query(
            "SELECT id, collection_id, title, tags, file_path, last_modified
             FROM items WHERE collection_id = ?1",
            libsql::params![collection_id],
        ).await?;

        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(row_to_item(&row)?);
        }
        Ok(items)
    }
}

fn row_to_item(row: &libsql::Row) -> anyhow::Result<ItemRow> {
    let tags_str: String = row.get(3)?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
    Ok(ItemRow {
        id: row.get(0)?,
        collection_id: row.get(1)?,
        title: row.get(2)?,
        tags,
        file_path: row.get(4)?,
        last_modified: row.get(5)?,
    })
}
