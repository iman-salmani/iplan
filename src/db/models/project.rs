use std::fmt;
use rusqlite::{Result, Row};

pub struct Project {
    pub id: i64,
    pub name: String,
    pub archive: bool,
    pub index: i64,
}

impl Project {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            archive: row.get(2)?,
            index: row.get(3)?,
        })
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Project(id: {}, name: {}, archive: {})",
            self.id, self.name, self.archive
        )
    }
}

