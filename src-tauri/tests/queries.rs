use kinleaf::{
    model::{EntryFields, PersonChoice, PersonFields},
    store::Store,
};

#[test]
fn search_paths_preserve_distinct_registrations() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path());
    let family = store.create_family("Test".into(), "Root".into()).unwrap();
    let id = &family.id;
    let root = &family.root_entry_id;
    let fields = EntryFields {
        rank: Some(2),
        branch_notes: None,
    };
    let add = |parent: &str, name: &str| {
        store
            .add_child(
                id,
                parent,
                PersonChoice::New(PersonFields {
                    name: name.into(),
                    ..Default::default()
                }),
                &fields,
            )
            .unwrap()
    };
    let left = add(root, "Left");
    let right = add(root, "Right");
    let first = add(&left, "Same");
    let shared = store.get_entry(id, &first).unwrap().hit.entry.person_id;
    let second = store
        .add_child(id, &right, PersonChoice::Existing(shared), &fields)
        .unwrap();
    let unrelated = add(root, "Same");
    let hits = store.search_entries(id, "Same").unwrap();
    assert_eq!(hits.len(), 3);
    for (entry_id, path) in [
        (&first, vec![root, &left, &first]),
        (&second, vec![root, &right, &second]),
        (&unrelated, vec![root, &unrelated]),
    ] {
        let hit = hits
            .iter()
            .find(|hit| hit.entry.entry_id == *entry_id)
            .unwrap();
        assert_eq!(hit.generation, path.len());
        assert_eq!(
            hit.path
                .iter()
                .map(|item| &item.entry_id)
                .collect::<Vec<_>>(),
            path
        );
    }
    let detail = store.get_entry(id, &first).unwrap();
    assert_eq!(detail.other_registrations[0].entry.entry_id, second);
    assert!(detail.father.is_none());
    let children = store.list_children(id, root).unwrap();
    assert!(children
        .windows(2)
        .all(|pair| pair[0].entry_id < pair[1].entry_id));
    assert!(
        children
            .iter()
            .find(|child| child.entry_id == left)
            .unwrap()
            .has_children
    );
}

#[test]
fn search_treats_wildcards_literally() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path());
    let family = store.create_family("Test".into(), "Root".into()).unwrap();
    let entry = store
        .add_child(
            &family.id,
            &family.root_entry_id,
            PersonChoice::New(PersonFields {
                name: "Name".into(),
                spectrum_name: Some("Alias%_".into()),
                ..Default::default()
            }),
            &EntryFields::default(),
        )
        .unwrap();
    for query in ["Alias", "%", "_"] {
        let hits = store.search_entries(&family.id, query).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.entry_id, entry);
    }
    assert!(store.search_entries(&family.id, "  ").unwrap().is_empty());
}
