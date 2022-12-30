
#[derive(Debug)]
pub struct ListItem {
    pub id: u32,
    pub added: sqlx::types::time::Date,
    pub title: String
}

#[derive(Debug)]
pub struct Item {
    pub id: u32,
    pub body: String,
    pub title: String,
    pub is_last: bool,
    pub is_first: bool,
}