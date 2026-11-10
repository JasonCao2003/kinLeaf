use std::io;

use rusqlite::{params, Connection, TransactionBehavior};
use uuid::Uuid;

use crate::{
    model::{EntryFields, PersonChoice, PersonFields, PersonSummary},
    store::{Store, StoreResult},
};

fn require(condition: bool, message: &'static str) -> StoreResult<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, message).into())
    }
}

fn insert_person(connection: &Connection, fields: PersonFields) -> StoreResult<String> {
    require(!fields.name.trim().is_empty(), "Name is required")?;
    let id = Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO person VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            fields.name.trim(),
            fields.spectrum_name,
            fields.generation_name,
            fields.birth_text,
            fields.death_text,
            fields.notes
        ],
    )?;
    Ok(id)
}

fn require_person(connection: &Connection, id: &str) -> StoreResult<()> {
    require(
        connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM person WHERE id = ?1)",
            [id],
            |row| row.get(0),
        )?,
        "Person not found",
    )
}

impl Store {
    pub fn create_person(&self, family: &str, fields: PersonFields) -> StoreResult<String> {
        insert_person(&self.open_family(family)?, fields)
    }

    pub fn update_person(&self, family: &str, id: &str, fields: PersonFields) -> StoreResult<()> {
        require(!fields.name.trim().is_empty(), "Name is required")?;
        let count = self.open_family(family)?.execute(
            "UPDATE person SET name = ?2, spectrum_name = ?3, generation_name = ?4,
             birth_text = ?5, death_text = ?6, notes = ?7 WHERE id = ?1",
            params![
                id,
                fields.name.trim(),
                fields.spectrum_name,
                fields.generation_name,
                fields.birth_text,
                fields.death_text,
                fields.notes
            ],
        )?;
        require(count == 1, "Person not found")
    }

    pub fn find_people(&self, family: &str, query: &str) -> StoreResult<Vec<PersonSummary>> {
        let connection = self.open_family(family)?;
        let mut statement = connection.prepare(
            "SELECT id, name, spectrum_name FROM person
             WHERE instr(name, ?1) > 0 OR instr(spectrum_name, ?1) > 0 ORDER BY name, id",
        )?;
        let people = statement
            .query_map([query.trim()], |row| {
                Ok(PersonSummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    spectrum_name: row.get(2)?,
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(people)
    }

    pub fn add_child(
        &self,
        family: &str,
        parent: &str,
        person: PersonChoice,
        fields: &EntryFields,
    ) -> StoreResult<String> {
        let mut connection = self.open_family(family)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let person_id = match person {
            PersonChoice::New(fields) => insert_person(&transaction, fields)?,
            PersonChoice::Existing(id) => id,
        };
        let conflict: bool = transaction.query_row(
            "WITH RECURSIVE ancestors(id, parent_entry_id, person_id) AS (
                SELECT id, parent_entry_id, person_id FROM entry WHERE id = ?1
                UNION SELECT e.id, e.parent_entry_id, e.person_id FROM entry e
                    JOIN ancestors a ON e.id = a.parent_entry_id
             ) SELECT EXISTS(SELECT 1 FROM ancestors WHERE person_id = ?2)",
            params![parent, person_id],
            |row| row.get(0),
        )?;
        require(!conflict, "Person already appears in the ancestor path")?;
        let id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO entry VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, person_id, parent, fields.rank, fields.branch_notes],
        )?;
        transaction.commit()?;
        Ok(id)
    }

    pub fn update_entry(
        &self,
        family: &str,
        id: &str,
        parent: Option<&str>,
        fields: &EntryFields,
    ) -> StoreResult<()> {
        let mut connection = self.open_family(family)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current_parent: Option<String> = transaction.query_row(
            "SELECT parent_entry_id FROM entry WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        require(
            current_parent.is_some() == parent.is_some(),
            "The founder must remain the only root",
        )?;
        let conflict: bool = transaction.query_row(
            "WITH RECURSIVE ancestors(id, parent_entry_id, person_id) AS (
                SELECT id, parent_entry_id, person_id FROM entry WHERE id = ?2
                UNION SELECT e.id, e.parent_entry_id, e.person_id FROM entry e
                    JOIN ancestors a ON e.id = a.parent_entry_id
             ), descendants(id, person_id) AS (
                SELECT id, person_id FROM entry WHERE id = ?1
                UNION SELECT e.id, e.person_id FROM entry e JOIN descendants d ON e.parent_entry_id = d.id
             ) SELECT EXISTS(SELECT 1 FROM ancestors JOIN descendants USING (person_id))",
            params![id, parent], |row| row.get(0),
        )?;
        require(
            !conflict,
            "The move would repeat a person in the ancestor path",
        )?;
        transaction.execute(
            "UPDATE entry SET parent_entry_id = ?2, rank = ?3, branch_notes = ?4 WHERE id = ?1",
            params![id, parent, fields.rank, fields.branch_notes],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn delete_entry(&self, family: &str, id: &str) -> StoreResult<()> {
        let count = self.open_family(family)?.execute(
            "DELETE FROM entry WHERE id = ?1 AND parent_entry_id IS NOT NULL
             AND NOT EXISTS (SELECT 1 FROM entry child WHERE child.parent_entry_id = ?1)",
            [id],
        )?;
        require(
            count == 1,
            "Only registrations without children can be deleted",
        )
    }

    pub fn set_parents(
        &self,
        family: &str,
        child: &str,
        father: Option<&str>,
        mother: Option<&str>,
    ) -> StoreResult<()> {
        let mut connection = self.open_family(family)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        require(
            father.is_none() || father != mother,
            "Parents must be distinct",
        )?;
        let conflict: bool = transaction.query_row(
            "WITH RECURSIVE ancestors(id) AS (
                SELECT ?2 WHERE ?2 IS NOT NULL UNION SELECT ?3 WHERE ?3 IS NOT NULL
                UNION SELECT p.father_id FROM parents p JOIN ancestors a ON p.child_id = a.id WHERE p.father_id IS NOT NULL
                UNION SELECT p.mother_id FROM parents p JOIN ancestors a ON p.child_id = a.id WHERE p.mother_id IS NOT NULL
             ) SELECT EXISTS(SELECT 1 FROM ancestors WHERE id = ?1)",
            params![child, father, mother], |row| row.get(0),
        )?;
        require(!conflict, "Biological ancestry cannot contain a cycle")?;
        transaction.execute(
            "INSERT INTO parents VALUES (?1, ?2, ?3) ON CONFLICT(child_id) DO UPDATE
             SET father_id = excluded.father_id, mother_id = excluded.mother_id",
            params![child, father, mother],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn add_new_spouse(
        &self,
        family: &str,
        person: &str,
        fields: PersonFields,
    ) -> StoreResult<String> {
        let mut connection = self.open_family(family)?;
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        require_person(&transaction, person)?;
        let spouse = insert_person(&transaction, fields)?;
        change_spouse_link(&transaction, person, &spouse, true)?;
        transaction.commit()?;
        Ok(spouse)
    }

    pub fn set_spouse(
        &self,
        family: &str,
        person: &str,
        other: &str,
        linked: bool,
    ) -> StoreResult<()> {
        require(person != other, "A person cannot be their own spouse")?;
        let connection = self.open_family(family)?;
        require_person(&connection, person)?;
        require_person(&connection, other)?;
        change_spouse_link(&connection, person, other, linked)
    }
}

fn change_spouse_link(
    connection: &Connection,
    person: &str,
    other: &str,
    linked: bool,
) -> StoreResult<()> {
    let (first, second) = (person.min(other), person.max(other));
    let sql = if linked {
        "INSERT INTO spouse VALUES (?1, ?2) ON CONFLICT DO NOTHING"
    } else {
        "DELETE FROM spouse WHERE person_a_id = ?1 AND person_b_id = ?2"
    };
    connection.execute(sql, params![first, second])?;
    Ok(())
}

impl Store {
    pub fn list_registrations(
        &self,
        family: &str,
    ) -> StoreResult<Vec<crate::model::EditableRegistration>> {
        let connection = self.open_family(family)?;
        let mut statement = connection.prepare(
            "SELECT e.id, e.person_id, e.parent_entry_id, p.name, e.rank, e.branch_notes
             FROM entry e JOIN person p ON p.id = e.person_id
             ORDER BY e.parent_entry_id IS NOT NULL, p.name, e.id",
        )?;
        let registrations = statement
            .query_map([], |row| {
                Ok(crate::model::EditableRegistration {
                    id: row.get(0)?,
                    person_id: row.get(1)?,
                    parent_entry_id: row.get(2)?,
                    name: row.get(3)?,
                    fields: EntryFields {
                        rank: row.get(4)?,
                        branch_notes: row.get(5)?,
                    },
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(registrations)
    }

    pub fn read_member(&self, family: &str, id: &str) -> StoreResult<crate::model::EditableMember> {
        let mut connection = self.open_family(family)?;
        let transaction = connection.transaction()?;
        read_member_on(&transaction, id)
    }
}

pub(crate) fn read_member_on(
    connection: &Connection,
    id: &str,
) -> StoreResult<crate::model::EditableMember> {
    let mut member = connection.query_row(
        "SELECT p.name, p.spectrum_name, p.generation_name, p.birth_text, p.death_text,
                    p.notes, r.father_id, r.mother_id,
                    (SELECT count(*) FROM entry WHERE person_id = p.id)
             FROM person p LEFT JOIN parents r ON p.id = r.child_id WHERE p.id = ?1",
        [id],
        |row| {
            Ok(crate::model::EditableMember {
                fields: PersonFields {
                    name: row.get(0)?,
                    spectrum_name: row.get(1)?,
                    generation_name: row.get(2)?,
                    birth_text: row.get(3)?,
                    death_text: row.get(4)?,
                    notes: row.get(5)?,
                },
                father_id: row.get(6)?,
                mother_id: row.get(7)?,
                registration_count: row.get(8)?,
                spouse_ids: Vec::new(),
            })
        },
    )?;
    let mut statement = connection.prepare(
        "SELECT person_b_id FROM spouse WHERE person_a_id = ?1
             UNION SELECT person_a_id FROM spouse WHERE person_b_id = ?1",
    )?;
    member.spouse_ids = statement
        .query_map([id], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    Ok(member)
}
