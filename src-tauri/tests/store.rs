use kinleaf::store::Store;

#[test]
fn create_reopen_and_ignore_failed_creation() {
    let directory = tempfile::tempdir().expect("create temporary family directory");
    let store = Store::new(directory.path());
    let created = store
        .create_family("曹氏族谱".to_owned(), "始祖".to_owned())
        .expect("create family");

    let reopened = Store::new(directory.path());
    let families = reopened
        .list_families()
        .expect("list families after reopening");
    assert_eq!(families.len(), 1);
    assert_eq!(families[0].id, created.id);
    assert_eq!(families[0].name, "曹氏族谱");
    assert_eq!(families[0].root_entry_id, created.root_entry_id);
    assert!(!created.root_entry_id.is_empty());

    assert!(reopened
        .create_family("另一份族谱".to_owned(), "  ".to_owned())
        .is_err());
    std::fs::write(
        directory.path().join("unfinished.sqlite3.tmp"),
        b"unfinished",
    )
    .expect("leave an unpublished file in the store directory");

    let families = reopened.list_families().expect("list published families");
    assert_eq!(families.len(), 1);
    assert_eq!(families[0].id, created.id);
}

#[test]
fn lists_healthy_families_when_another_file_is_corrupt() {
    let directory = tempfile::tempdir().expect("create temporary family directory");
    let store = Store::new(directory.path());
    let healthy = store
        .create_family("Healthy".into(), "Founder".into())
        .unwrap();
    let damaged_id = uuid::Uuid::new_v4();
    std::fs::write(
        directory.path().join(format!("{damaged_id}.sqlite3")),
        b"damaged",
    )
    .unwrap();

    let catalog = store.read_catalog().expect("read healthy families");
    assert_eq!(catalog.families.len(), 1);
    assert_eq!(catalog.families[0].id, healthy.id);
    assert_eq!(catalog.unreadable_count, 1);
}
