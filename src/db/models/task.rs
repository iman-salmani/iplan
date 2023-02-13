use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct Task {
        pub id: Cell<i64>,
        pub name: RefCell<String>,
        pub done: Cell<bool>,
        pub project: Cell<i64>,
        pub list: Cell<i64>,
        pub duration: RefCell<String>,
        pub position: Cell<i32>,
        pub suspended: Cell<bool>,
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
                    glib::ParamSpecString::builder("duration").build(),
                    glib::ParamSpecInt::builder("position").build(),
                    glib::ParamSpecBoolean::builder("suspended").build(),
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
                "duration" => {
                    let value = value.get::<String>().expect("Value must be a String");
                    self.duration.replace(value);
                }
                "position" => {
                    let value = value.get::<i32>().expect("Value must be a i32");
                    self.position.set(value);
                }
                "suspended" => {
                    let value = value.get::<bool>().expect("Value must be a bool");
                    self.suspended.set(value);
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
                "duration" => self.duration.borrow().to_string().to_value(),
                "position" => self.position.get().to_value(),
                "suspended" => self.suspended.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Task(ObjectSubclass<imp::Task>);
}

impl Task {
    pub fn new(
        id: i64,
        name: String,
        done: bool,
        project: i64,
        list: i64,
        duration: String,
        position: i32,
        suspended: bool,
    ) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("done", done)
            .property("project", project)
            .property("list", list)
            .property("duration", duration)
            .property("position", position)
            .property("suspended", suspended)
            .build()
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

    pub fn duration(&self) -> String {
        self.property("duration")
    }

    pub fn position(&self) -> i32 {
        self.property("position")
    }

    pub fn suspended(&self) -> bool {
        self.property("suspended")
    }
}

impl TryFrom<&Row<'_>> for Task {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Task::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
            row.get(6)?,
            row.get(7)?,
        ))
    }
}

impl Default for Task {
    fn default() -> Self {
        Task::new(1, String::new(), false, 1, 1, String::new(), 0, false)
    }
}
