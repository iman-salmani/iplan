use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

use crate::db::models::Record;
use crate::db::operations::{read_records, read_tasks};

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct Task {
        pub id: Cell<i64>,
        pub name: RefCell<String>,
        pub done: Cell<bool>,
        pub project: Cell<i64>,
        pub list: Cell<i64>,
        pub position: Cell<i32>,
        pub suspended: Cell<bool>,
        pub parent: Cell<i64>,
        pub description: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Task {
        const NAME: &'static str = "Task";
        type Type = super::Task;
    }

    impl ObjectImpl for Task {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").build(),
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecBoolean::builder("done").build(),
                    glib::ParamSpecInt64::builder("project").build(),
                    glib::ParamSpecInt64::builder("list").build(),
                    glib::ParamSpecInt::builder("position").build(),
                    glib::ParamSpecBoolean::builder("suspended").build(),
                    glib::ParamSpecInt64::builder("parent").build(),
                    glib::ParamSpecString::builder("description").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "id" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.id.set(value);
                }
                "name" => {
                    let value = value.get::<String>().expect("Value must be a String");
                    self.name.replace(value);
                }
                "done" => {
                    let value = value.get::<bool>().expect("Value must be a bool");
                    self.done.set(value);
                }
                "project" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.project.set(value);
                }
                "list" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.list.set(value);
                }
                "position" => {
                    let value = value.get::<i32>().expect("Value must be a i32");
                    self.position.set(value);
                }
                "suspended" => {
                    let value = value.get::<bool>().expect("Value must be a bool");
                    self.suspended.set(value);
                }
                "parent" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.parent.set(value);
                }
                "description" => {
                    let value = value.get::<String>().expect("Value must be a String");
                    self.description.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "name" => self.name.borrow().to_string().to_value(),
                "done" => self.done.get().to_value(),
                "project" => self.project.get().to_value(),
                "list" => self.list.get().to_value(),
                "position" => self.position.get().to_value(),
                "suspended" => self.suspended.get().to_value(),
                "parent" => self.parent.get().to_value(),
                "description" => self.description.borrow().to_string().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Task(ObjectSubclass<imp::Task>);
}

impl Task {
    pub fn new(properties: &[(&str, &dyn ToValue)]) -> Self {
        glib::Object::new::<Self>(properties)
    }

    pub fn duration(&self) -> i64 {
        let mut total = 0;
        for record in read_records(self.id(), false, None, None).expect("Failed to read records") {
            total += record.duration();
        }
        for subtask in read_tasks(self.project(), None, None, Some(self.id()))
            .expect("Failed to read subtasks")
        {
            total += subtask.duration();
        }
        total
    }

    pub fn duration_display(&self) -> String {
        Record::duration_display(self.duration())
    }

    pub fn id(&self) -> i64 {
        self.property("id")
    }

    pub fn name(&self) -> String {
        self.property("name")
    }

    pub fn done(&self) -> bool {
        self.property("done")
    }

    pub fn project(&self) -> i64 {
        self.property("project")
    }

    pub fn list(&self) -> i64 {
        self.property("list")
    }

    pub fn position(&self) -> i32 {
        self.property("position")
    }

    pub fn suspended(&self) -> bool {
        self.property("suspended")
    }

    pub fn parent(&self) -> i64 {
        self.property("parent")
    }

    pub fn description(&self) -> String {
        self.property("description")
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
            ("list", &row.get::<usize, i64>(4)?),
            ("position", &row.get::<usize, i32>(5)?),
            ("suspended", &row.get::<usize, bool>(6)?),
            ("parent", &row.get::<usize, i64>(7)?),
            ("description", &row.get::<usize, String>(8)?),
        ]))
    }
}

impl Default for Task {
    fn default() -> Self {
        Task::new(&[])
    }
}
