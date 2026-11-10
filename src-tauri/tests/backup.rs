use kinleaf::{
    backup::{export_to, restore_from},
    store::Store,
};
use rusqlite::Connection;

#[test]
fn restore_accepts_schema_created_with_windows_line_endings() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(&directory.path().join("families"));
    let backup = directory.path().join("windows.sqlite3");
    let database = Connection::open(&backup).unwrap();
    let migration = include_str!("../migrations/001_init.sql")
        .replace("\r\n", "\n")
        .replace('\n', "\r\n");
    database.execute_batch(&migration).unwrap();
    database
        .execute_batch(
            "INSERT INTO person (id, name) VALUES ('person', 'Founder');
         INSERT INTO entry (id, person_id) VALUES ('root', 'person');
         INSERT INTO family VALUES ('original', 'Family', 'root');",
        )
        .unwrap();
    drop(database);
    let restored = restore_from(&store, &backup).unwrap();
    assert_eq!(restored.name, "Family");
    assert_ne!(restored.id, "original");
}

#[test]
fn snapshot_includes_wal_and_restores_an_independent_family() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(&directory.path().join("families"));
    let original = store
        .create_family("Original".into(), "Founder".into())
        .unwrap();
    let writer = Connection::open(
        directory
            .path()
            .join("families")
            .join(format!("{}.sqlite3", original.id)),
    )
    .unwrap();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    writer
        .execute("UPDATE person SET name = 'Snapshot'", [])
        .unwrap();
    let backup = directory.path().join("copy.sqlite3");
    export_to(&store, &original.id, &backup).unwrap();
    writer
        .execute("UPDATE person SET name = 'Changed'", [])
        .unwrap();
    let restored = restore_from(&store, &backup).unwrap();
    assert_ne!(original.id, restored.id);
    assert_eq!(original.root_entry_id, restored.root_entry_id);
    assert_eq!(
        store.find_people(&restored.id, "Snapshot").unwrap().len(),
        1
    );
    assert_eq!(store.find_people(&original.id, "Changed").unwrap().len(), 1);
    assert_eq!(store.list_families().unwrap().len(), 2);
}

#[test]
fn invalid_backups_and_existing_destinations_are_preserved() {
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(&directory.path().join("families"));
    let original = store
        .create_family("Original".into(), "Founder".into())
        .unwrap();
    let backup = directory.path().join("copy.sqlite3");
    std::fs::write(&backup, b"existing").unwrap();
    let misplaced = directory
        .path()
        .join("families")
        .join(format!("{}.sqlite3", uuid::Uuid::new_v4()));
    assert!(export_to(&store, &original.id, &misplaced).is_err());
    assert!(!misplaced.exists());
    assert!(export_to(&store, &original.id, &backup).is_err());
    assert_eq!(std::fs::read(&backup).unwrap(), b"existing");
    assert!(restore_from(&store, &backup).is_err());
    for (index, alteration) in [
        "PRAGMA user_version=99",
        "INSERT INTO entry VALUES ('orphan','absent','missing',NULL,NULL)",
        "INSERT INTO entry SELECT 'loop',person_id,'loop',NULL,NULL FROM entry",
        "INSERT INTO parents SELECT person_id,person_id,NULL FROM entry",
    ]
    .iter()
    .enumerate()
    {
        let invalid = directory.path().join(format!("invalid-{index}.sqlite3"));
        export_to(&store, &original.id, &invalid).unwrap();
        let database = Connection::open(&invalid).unwrap();
        database.pragma_update(None, "foreign_keys", false).unwrap();
        database.execute_batch(alteration).unwrap();
        drop(database);
        assert!(restore_from(&store, &invalid).is_err(), "{alteration}");
    }
    assert_eq!(store.list_families().unwrap()[0].id, original.id);
    assert_eq!(
        std::fs::read_dir(directory.path().join("families"))
            .unwrap()
            .count(),
        1
    );
}
