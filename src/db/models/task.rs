use rusqlite::{Error, Result, Row};
use std::fmt;

pub struct Task {
    pub id: i64,
    pub name: String,
    pub done: bool,
    pub project: i64,
    pub list: i64,
    pub duration: String,
    pub position: i64,
    pub suspended: bool,
}

impl Task {
    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Task {
            id: row.get(0)?,
            name: row.get(1)?,
            done: row.get(2)?,
            project: row.get(3)?,
            list: row.get(4)?,
            duration: row.get(5)?,
            position: row.get(6)?,
            suspended: row.get(7)?,
        })
    }
}

// impl TryFrom<&Row<'_>> for Task {
//     type Error = Error;

//     fn try_from(row: &Row) -> Result<Self, Self::Error> {
//         Ok(Task::new(
//             row.get(0)?,
//             row.get(1)?,
//             row.get(2)?,
//             row.get(3)?,
//         ))
//     }
// }

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Task(id: {}, name: {}, project: {}, list: {})",
            self.id, self.name, self.project, self.list
        )
    }
}
