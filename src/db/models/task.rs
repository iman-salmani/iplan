use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

use crate::db::models::Record;
use crate::db::operations::{read_records, read_tasks};

mod imp {
    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Task)]
    pub struct Task {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub done: Cell<bool>,
        #[property(get, set)]
        pub project: Cell<i64>,
        #[property(get, set)]
        pub section: Cell<i64>,
        #[property(get, set)]
        pub position: Cell<i32>,
        #[property(get, set)]
        pub suspended: Cell<bool>,
        #[property(get, set)]
        pub parent: Cell<i64>,
        #[property(get, set)]
        pub description: RefCell<String>,
        #[property(get, set)]
        pub date: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Task {
        const NAME: &'static str = "Task";
        type Type = super::Task;
    }

    impl ObjectImpl for Task {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct Task(ObjectSubclass<imp::Task>);
}

impl Task {
    pub fn new(properties: &[(&str, &dyn ToValue)]) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_properties(properties);
        obj
    }

    pub fn duration(&self) -> i64 {
        let mut total = 0;
        for record in read_records(self.id(), false, None, None).expect("Failed to read records") {
            total += record.duration();
        }
        for subtask in read_tasks(Some(self.project()), None, None, Some(self.id()), None)
            .expect("Failed to read subtasks")
        {
            total += subtask.duration();
        }
        total
    }

    pub fn duration_display(&self) -> String {
        Record::duration_display(self.duration())
    }

    pub fn incomplete_record(&self) -> Option<Record> {
        let incomplete_records =
            read_records(self.id(), true, None, None).expect("Failed to read records");
        match incomplete_records.len() {
            0 => None,
            1 => {
                let record = incomplete_records.get(0).unwrap().to_owned();
                record
                    .set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
                Some(record)
            }
            _ => panic!("The Task cannot have multiple incomplete records"),
        }
    }

    pub fn date_datetime(&self) -> Option<glib::DateTime> {
        let date = self.date();
        if date == 0 {
            None
        } else {
            Some(glib::DateTime::from_unix_local(self.date()).unwrap())
        }
    }
}

impl TryFrom<&Row<'_>> for Task {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Task::new(&[
            ("id", &row.get::<usize, i64>(0)?),
            ("name", &row.get::<usize, String>(1)?),
            ("done", &row.get::<usize, bool>(2)?),
            ("project", &row.get::<usize, i64>(3)?),
            ("section", &row.get::<usize, i64>(4)?),
            ("position", &row.get::<usize, i32>(5)?),
            ("suspended", &row.get::<usize, bool>(6)?),
            ("parent", &row.get::<usize, i64>(7)?),
            ("description", &row.get::<usize, String>(8)?),
            ("date", &row.get::<usize, i64>(9)?),
        ]))
    }
}

impl Default for Task {
    fn default() -> Self {
        Task::new(&[])
    }
}
