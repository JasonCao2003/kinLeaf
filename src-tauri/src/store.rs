use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use rusqlite::{params, Connection, OpenFlags};
use uuid::Uuid;

use crate::model::{FamilyCatalog, FamilySummary};

pub struct Store {
    directory: PathBuf,
}

type StoreResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

impl Store {
    pub fn new(directory: &Path) -> Self {
        Self {
            directory: directory.to_owned(),
        }
    }

    pub fn create_family(&self, name: String, founder_name: String) -> StoreResult<FamilySummary> {
        let name = name.trim();
        let founder_name = founder_name.trim();
        if name.is_empty() || founder_name.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Family and founder names are required",
            )
            .into());
        }

        fs::create_dir_all(&self.directory)?;
        let id = Uuid::new_v4().to_string();
        let person_id = Uuid::new_v4().to_string();
        let root_entry_id = Uuid::new_v4().to_string();
        let staging = tempfile::NamedTempFile::new_in(&self.directory)?.into_temp_path();
        let mut connection = Connection::open(&staging)?;
        connection.pragma_update(None, "foreign_keys", true)?;
        let transaction = connection.transaction()?;
        transaction.execute_batch(include_str!("../migrations/001_init.sql"))?;
        transaction.execute(
            "INSERT INTO person (id, name) VALUES (?1, ?2)",
            params![person_id, founder_name],
        )?;
        transaction.execute(
            "INSERT INTO entry (id, person_id) VALUES (?1, ?2)",
            params![root_entry_id, person_id],
        )?;
        transaction.execute(
            "INSERT INTO family VALUES (?1, ?2, ?3)",
            params![id, name, root_entry_id],
        )?;
        transaction.commit()?;
        connection.close().map_err(|(_, error)| error)?;
        staging.persist_noclobber(self.directory.join(format!("{id}.sqlite3")))?;

        Ok(FamilySummary {
            id,
            name: name.to_owned(),
            root_entry_id,
        })
    }

    pub fn list_families(&self) -> StoreResult<Vec<FamilySummary>> {
        Ok(self.read_catalog()?.families)
    }

    pub fn read_catalog(&self) -> StoreResult<FamilyCatalog> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(FamilyCatalog::default())
            }
            Err(error) => return Err(error.into()),
        };

        let mut catalog = FamilyCatalog::default();
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let published = path.extension().and_then(|ext| ext.to_str()) == Some("sqlite3")
                && path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| Uuid::parse_str(stem).is_ok());
            if !entry.file_type()?.is_file() || !published {
                continue;
            }

            let summary = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .and_then(|connection| {
                    connection.query_row("SELECT id, name, root_entry_id FROM family", [], |row| {
                        Ok(FamilySummary {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            root_entry_id: row.get(2)?,
                        })
                    })
                });
            match summary {
                Ok(summary) => catalog.families.push(summary),
                Err(_) => catalog.unreadable_count += 1,
            }
        }
        catalog.families.sort_by(|left, right| {
            left.name
                .cmp(&right.name)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(catalog)
    }
}
