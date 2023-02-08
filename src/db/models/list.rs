use std::fmt;
use rusqlite::{Result, Row};

pub struct List {
    pub id: i64,
    pub name: String,
    pub project: i64,
    pub index: u32,
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
            "List(id: {}, name: {}, project: {}, index: {})",
            self.id, self.name, self.project, self.index
        )
    }
}

