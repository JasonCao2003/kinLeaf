use kinleaf::{
    model::{EntryFields, PersonChoice, PersonFields},
    store::Store,
};

fn person(name: &str) -> PersonFields {
    PersonFields {
        name: name.into(),
        ..Default::default()
    }
}

#[test]
fn edits_registrations_and_rejects_invalid_relationships() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path());
    let family = store.create_family("Test".into(), "Root".into()).unwrap();
    let id = &family.id;
    let root = &family.root_entry_id;
    let fields = EntryFields::default();
    let person_id = store.create_person(id, person("Shared")).unwrap();
    let branch_person = store.create_person(id, person("Branch")).unwrap();
    let branch = store
        .add_child(
            id,
            root,
            PersonChoice::Existing(branch_person.clone()),
            &fields,
        )
        .unwrap();
    let first = store
        .add_child(id, root, PersonChoice::Existing(person_id.clone()), &fields)
        .unwrap();
    let second = store
        .add_child(
            id,
            &branch,
            PersonChoice::Existing(person_id.clone()),
            &EntryFields {
                rank: Some(2),
                branch_notes: Some("Second".into()),
            },
        )
        .unwrap();
    assert_ne!(first, second);
    assert!(store
        .add_child(id, "missing", PersonChoice::New(person("Orphan")), &fields)
        .is_err());
    assert!(store.find_people(id, "Orphan").unwrap().is_empty());
    assert!(store
        .update_entry(id, &branch, Some(&first), &fields)
        .is_err());
    assert!(store
        .update_entry(id, &branch, Some(&second), &fields)
        .is_err());
    assert!(store.delete_entry(id, &branch).is_err());
    assert!(store.delete_entry(id, root).is_err());
    store
        .update_person(id, &person_id, person("Renamed"))
        .unwrap();
    assert_eq!(store.find_people(id, "Renamed").unwrap()[0].id, person_id);

    let partner = store.create_person(id, person("Partner")).unwrap();
    store.set_spouse(id, &person_id, &partner, true).unwrap();
    store
        .set_spouse(id, &person_id, &branch_person, true)
        .unwrap();
    assert!(store.set_spouse(id, &person_id, &person_id, true).is_err());
    store
        .set_parents(id, &person_id, Some(&branch_person), None)
        .unwrap();
    assert!(store
        .set_parents(id, &branch_person, Some(&person_id), None)
        .is_err());
    let profile = store.read_member(id, &person_id).unwrap();
    assert_eq!(profile.registration_count, 2);
    assert_eq!(profile.father_id.as_deref(), Some(branch_person.as_str()));
    assert!(profile.mother_id.is_none());
    assert_eq!(profile.spouse_ids.len(), 2);

    let registrations = store.list_registrations(id).unwrap();
    assert_eq!(registrations.first().unwrap().id, *root);
    let duplicate = registrations
        .iter()
        .find(|entry| entry.id == second)
        .unwrap();
    assert_eq!(duplicate.person_id, person_id);
    assert_eq!(duplicate.fields.rank, Some(2));
    store.delete_entry(id, &first).unwrap();

    let database = rusqlite::Connection::open(dir.path().join(format!("{id}.sqlite3"))).unwrap();
    let spouse_count: i64 = database
        .query_row("SELECT count(*) FROM spouse", [], |row| row.get(0))
        .unwrap();
    assert_eq!(spouse_count, 2);
    let parent: String = database
        .query_row(
            "SELECT parent_entry_id FROM entry WHERE id = ?1",
            [&branch],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(parent, *root);
    let other: (String, i64, String) = database.query_row("SELECT person.name, entry.rank, entry.branch_notes FROM entry JOIN person ON person.id = entry.person_id WHERE entry.id = ?1", [&second], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).unwrap();
    assert_eq!(other, ("Renamed".into(), 2, "Second".into()));
}
