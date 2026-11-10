PRAGMA application_id = 1263095372;
PRAGMA user_version = 1;

CREATE TABLE person (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    spectrum_name TEXT,
    generation_name TEXT,
    birth_text TEXT,
    death_text TEXT,
    notes TEXT
);

CREATE TABLE entry (
    id TEXT PRIMARY KEY NOT NULL,
    person_id TEXT NOT NULL REFERENCES person(id),
    parent_entry_id TEXT REFERENCES entry(id),
    rank INTEGER CHECK (rank > 0),
    branch_notes TEXT
);

CREATE TABLE family (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    root_entry_id TEXT NOT NULL REFERENCES entry(id)
);

CREATE TABLE parents (
    child_id TEXT PRIMARY KEY NOT NULL REFERENCES person(id),
    father_id TEXT REFERENCES person(id),
    mother_id TEXT REFERENCES person(id)
);

CREATE TABLE spouse (
    person_a_id TEXT NOT NULL REFERENCES person(id),
    person_b_id TEXT NOT NULL REFERENCES person(id),
    PRIMARY KEY (person_a_id, person_b_id),
    CHECK (person_a_id < person_b_id)
);

CREATE INDEX entry_parent ON entry(parent_entry_id);
CREATE INDEX entry_person ON entry(person_id);
