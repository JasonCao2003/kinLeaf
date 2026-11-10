use std::{
    collections::{HashMap, HashSet},
    io,
};

use rusqlite::Connection;

use crate::{
    members::read_member_on,
    model::{EntryDetail, EntrySummary, PersonSummary, SearchHit},
    store::{Store, StoreResult},
};

struct Node {
    summary: EntrySummary,
    parent: Option<String>,
    alias: Option<String>,
}

fn read_nodes(connection: &Connection) -> StoreResult<HashMap<String, Node>> {
    let mut query = connection.prepare(
        "SELECT e.id, e.person_id, p.name, e.rank,
                EXISTS(SELECT 1 FROM entry c WHERE c.parent_entry_id = e.id),
                e.parent_entry_id, p.spectrum_name
         FROM entry e JOIN person p ON p.id = e.person_id",
    )?;
    let nodes = query
        .query_map([], |row| {
            let entry_id: String = row.get(0)?;
            Ok((
                entry_id.clone(),
                Node {
                    summary: EntrySummary {
                        entry_id,
                        person_id: row.get(1)?,
                        name: row.get(2)?,
                        rank: row.get(3)?,
                        has_children: row.get(4)?,
                    },
                    parent: row.get(5)?,
                    alias: row.get(6)?,
                },
            ))
        })?
        .collect::<Result<_, _>>()?;
    Ok(nodes)
}

fn hit_for(nodes: &HashMap<String, Node>, id: &str) -> StoreResult<SearchHit> {
    let mut current = Some(id);
    let mut path = Vec::new();
    let mut visited = HashSet::new();
    while let Some(entry_id) = current {
        if !visited.insert(entry_id) {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "Lineage contains a cycle").into(),
            );
        }
        let node = nodes
            .get(entry_id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Registration not found"))?;
        path.push(node.summary.clone());
        current = node.parent.as_deref();
    }
    path.reverse();
    let entry = path
        .last()
        .expect("the requested registration starts the path")
        .clone();
    Ok(SearchHit {
        entry,
        generation: path.len(),
        path,
    })
}

fn sort_hits(hits: &mut [SearchHit]) {
    hits.sort_by(|a, b| {
        a.generation
            .cmp(&b.generation)
            .then_with(|| a.entry.name.cmp(&b.entry.name))
            .then_with(|| a.entry.entry_id.cmp(&b.entry.entry_id))
    });
}

fn person_summary(connection: &Connection, id: &str) -> rusqlite::Result<PersonSummary> {
    connection.query_row(
        "SELECT id, name, spectrum_name FROM person WHERE id = ?1",
        [id],
        |row| {
            Ok(PersonSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                spectrum_name: row.get(2)?,
            })
        },
    )
}

impl Store {
    pub fn list_children(&self, family: &str, parent: &str) -> StoreResult<Vec<EntrySummary>> {
        let connection = self.open_family(family)?;
        let mut query = connection.prepare(
            "SELECT e.id, e.person_id, p.name, e.rank,
                    EXISTS(SELECT 1 FROM entry c WHERE c.parent_entry_id = e.id)
             FROM entry e JOIN person p ON p.id = e.person_id
             WHERE e.parent_entry_id = ?1 ORDER BY e.rank IS NULL, e.rank, e.id",
        )?;
        let children = query
            .query_map([parent], |row| {
                Ok(EntrySummary {
                    entry_id: row.get(0)?,
                    person_id: row.get(1)?,
                    name: row.get(2)?,
                    rank: row.get(3)?,
                    has_children: row.get(4)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(children)
    }

    pub fn search_entries(&self, family: &str, query: &str) -> StoreResult<Vec<SearchHit>> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let nodes = read_nodes(&self.open_family(family)?)?;
        let mut hits = nodes
            .values()
            .filter(|node| {
                node.summary.name.contains(query)
                    || node
                        .alias
                        .as_deref()
                        .is_some_and(|alias| alias.contains(query))
            })
            .map(|node| hit_for(&nodes, &node.summary.entry_id))
            .collect::<StoreResult<Vec<_>>>()?;
        sort_hits(&mut hits);
        Ok(hits)
    }

    pub fn get_entry(&self, family: &str, id: &str) -> StoreResult<EntryDetail> {
        let mut connection = self.open_family(family)?;
        let snapshot = connection.transaction()?;
        let nodes = read_nodes(&snapshot)?;
        let hit = hit_for(&nodes, id)?;
        let profile = read_member_on(&snapshot, &hit.entry.person_id)?;
        let fields = snapshot.query_row(
            "SELECT rank, branch_notes FROM entry WHERE id = ?1",
            [id],
            |row| {
                Ok(crate::model::EntryFields {
                    rank: row.get(0)?,
                    branch_notes: row.get(1)?,
                })
            },
        )?;
        let mut other_registrations = nodes
            .values()
            .filter(|node| {
                node.summary.person_id == hit.entry.person_id && node.summary.entry_id != id
            })
            .map(|node| hit_for(&nodes, &node.summary.entry_id))
            .collect::<StoreResult<Vec<_>>>()?;
        sort_hits(&mut other_registrations);
        Ok(EntryDetail {
            hit,
            fields,
            person: profile.fields,
            father: profile
                .father_id
                .as_deref()
                .map(|id| person_summary(&snapshot, id))
                .transpose()?,
            mother: profile
                .mother_id
                .as_deref()
                .map(|id| person_summary(&snapshot, id))
                .transpose()?,
            spouses: profile
                .spouse_ids
                .iter()
                .map(|id| person_summary(&snapshot, id))
                .collect::<Result<_, _>>()?,
            other_registrations,
        })
    }
}
