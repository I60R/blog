
#[derive(Debug)]
pub struct ListItem {
    pub added: sqlx::types::time::Date,
    pub title: String
}

#[derive(Debug)]
pub struct Item {
    pub body: String,
    pub title: String,
    pub id: u32,
    pub is_last: bool,
    pub is_first: bool,
}