use std::{fs, io, path::Path};

use rusqlite::{
    backup::{Backup, StepResult},
    Connection, OpenFlags,
};
use uuid::Uuid;

use crate::{
    model::FamilySummary,
    store::{Store, StoreResult},
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn copy_snapshot(source: &Connection, destination: &mut Connection) -> StoreResult<()> {
    match Backup::new(source, destination)?.step(-1)? {
        StepResult::Done => Ok(()),
        _ => Err(io::Error::new(io::ErrorKind::WouldBlock, "Database is busy").into()),
    }
}

pub fn export_to(store: &Store, family_id: &str, destination: &Path) -> StoreResult<()> {
    let directory = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let source = store.open_family(family_id)?;
    if directory.canonicalize()? == store.directory().canonicalize()? {
        return Err(invalid("Save backups outside the managed families directory").into());
    }
    let staged = tempfile::NamedTempFile::new_in(directory)?.into_temp_path();
    let mut snapshot = Connection::open(&staged)?;
    copy_snapshot(&source, &mut snapshot)?;
    snapshot.close().map_err(|(_, error)| error)?;
    staged.persist_noclobber(destination)?;
    Ok(())
}

pub fn restore_from(store: &Store, source: &Path) -> StoreResult<FamilySummary> {
    let source = Connection::open_with_flags(source, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    fs::create_dir_all(store.directory())?;
    let staged = tempfile::NamedTempFile::new_in(store.directory())?.into_temp_path();
    let mut snapshot = Connection::open(&staged)?;
    copy_snapshot(&source, &mut snapshot)?;
    snapshot.pragma_update(None, "trusted_schema", false)?;
    snapshot.pragma_update(None, "foreign_keys", true)?;
    let mut family = validate_snapshot(&snapshot)?;
    let new_id = Uuid::new_v4().to_string();
    snapshot.execute(
        "UPDATE family SET id = ?1 WHERE id = ?2",
        [&new_id, &family.id],
    )?;
    family.id = new_id;
    snapshot.close().map_err(|(_, error)| error)?;
    staged.persist_noclobber(store.directory().join(format!("{}.sqlite3", family.id)))?;
    Ok(family)
}

fn validate_snapshot(db: &Connection) -> StoreResult<FamilySummary> {
    let application: i64 = db.pragma_query_value(None, "application_id", |row| row.get(0))?;
    let version: i64 = db.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if application != 1263095372 || version != 1 {
        return Err(invalid("Unsupported Kinleaf backup format or version").into());
    }

    // Compare the schema as well as its version before opening an externally supplied file as a family.
    let reference = Connection::open_in_memory()?;
    reference.execute_batch(include_str!("../migrations/001_init.sql"))?;
    if schema(db)? != schema(&reference)? {
        return Err(invalid("Unexpected backup schema").into());
    }

    let integrity: String = db.pragma_query_value(None, "integrity_check", |row| row.get(0))?;
    if integrity != "ok" || exists(db, "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_check)")? {
        return Err(invalid("Backup failed database integrity checks").into());
    }
    if !exists(db, "SELECT count(*) = 1 FROM family")? {
        return Err(invalid("Backup must contain one family").into());
    }
    let family = db.query_row("SELECT id, name, root_entry_id FROM family", [], |row| {
        Ok(FamilySummary {
            id: row.get(0)?,
            name: row.get(1)?,
            root_entry_id: row.get(2)?,
        })
    })?;
    let lineage_valid = exists(db, "
        WITH RECURSIVE reach(id) AS (
            SELECT e.id FROM family f JOIN entry e ON e.id = f.root_entry_id
                WHERE e.parent_entry_id IS NULL
            UNION SELECT e.id FROM entry e JOIN reach r ON e.parent_entry_id = r.id
        ), ancestors(child, ancestor) AS (
            SELECT id, parent_entry_id FROM entry WHERE parent_entry_id IS NOT NULL
            UNION SELECT a.child, e.parent_entry_id FROM ancestors a JOIN entry e ON e.id = a.ancestor
                WHERE e.parent_entry_id IS NOT NULL
        )
        SELECT (SELECT count(*) FROM reach) = (SELECT count(*) FROM entry)
            AND EXISTS (SELECT 1 FROM reach)
            AND NOT EXISTS (
                SELECT 1 FROM ancestors a JOIN entry c ON c.id = a.child
                    JOIN entry p ON p.id = a.ancestor WHERE c.person_id = p.person_id
            )")?;
    let invalid_parents = exists(
        db,
        "
        WITH RECURSIVE links(child, parent) AS (
            SELECT child_id, father_id FROM parents WHERE father_id IS NOT NULL
            UNION SELECT child_id, mother_id FROM parents WHERE mother_id IS NOT NULL
        ), ancestry(child, ancestor) AS (
            SELECT child, parent FROM links
            UNION SELECT a.child, l.parent FROM ancestry a JOIN links l ON l.child = a.ancestor
        )
        SELECT EXISTS(SELECT 1 FROM ancestry WHERE child = ancestor)
            OR EXISTS(SELECT 1 FROM parents WHERE father_id = mother_id)",
    )?;
    if !lineage_valid || invalid_parents {
        return Err(invalid("Backup contains invalid lineage or parent relationships").into());
    }
    Ok(family)
}

fn exists(db: &Connection, sql: &str) -> rusqlite::Result<bool> {
    db.query_row(sql, [], |row| row.get(0))
}

fn schema(db: &Connection) -> StoreResult<Vec<(String, String, String)>> {
    let mut statement = db.prepare(
        "SELECT type, name, sql FROM sqlite_schema WHERE sql IS NOT NULL ORDER BY type, name",
    )?;
    let rows = statement.query_map([], |row| {
        let sql: String = row.get(2)?;
        Ok((row.get(0)?, row.get(1)?, sql.replace("\r\n", "\n")))
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}
