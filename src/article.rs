use sqlx::types::time::Date;

#[derive(Debug)]
pub struct ListItem {
    pub id: u32,
    pub added: Date,
    pub title: String
}

pub struct Item {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub is_last: bool,
    pub is_first: bool,
}


impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Item")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("#", &self.body.len())
            .field("is_last", &self.is_last)
            .field("is_first", &self.is_first)
            .finish()
    }
}