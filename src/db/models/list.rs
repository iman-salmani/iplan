use rusqlite::{Result, Row};
use std::fmt;

pub struct List {
    pub id: i64,
    pub name: String,
    pub project: i64,
    pub index: i64,
}

impl List {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(List {
            id: row.get(0)?,
            name: row.get(1)?,
            project: row.get(2)?,
            index: row.get(3)?,
        })
    }
}

impl fmt::Display for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "List(id: {}, name: {}, project: {})",
            self.id, self.name, self.project
        )
    }
}
