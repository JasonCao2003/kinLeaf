use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilySummary {
    pub id: String,
    pub name: String,
    pub root_entry_id: String,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyCatalog {
    pub families: Vec<FamilySummary>,
    pub unreadable_count: usize,
}
