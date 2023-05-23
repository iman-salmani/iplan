use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

use crate::db::operations::read_tasks;

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct Project {
        pub id: Cell<i64>,
        pub name: RefCell<String>,
        pub archive: Cell<bool>,
        pub index: Cell<i32>,
        pub icon: RefCell<String>,
        pub description: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Project {
        const NAME: &'static str = "Project";
        type Type = super::Project;
    }

    impl ObjectImpl for Project {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").build(),
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecBoolean::builder("archive").build(),
                    glib::ParamSpecInt::builder("index").build(),
                    glib::ParamSpecString::builder("icon").build(),
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
                "archive" => {
                    let value = value.get::<bool>().expect("Value must be a bool");
                    self.archive.set(value);
                }
                "index" => {
                    let value = value.get::<i32>().expect("Value must be a i32");
                    self.index.set(value);
                }
                "icon" => {
                    let value = value.get::<String>().expect("Value must be a String");
                    self.icon.replace(value);
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
                "archive" => self.archive.get().to_value(),
                "index" => self.index.get().to_value(),
                "icon" => self.icon.borrow().to_string().to_value(),
                "description" => self.description.borrow().to_string().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct Project(ObjectSubclass<imp::Project>);
}

impl Project {
    pub fn new(
        id: i64,
        name: String,
        archive: bool,
        index: i32,
        icon: String,
        description: String,
    ) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("archive", archive)
            .property("index", index)
            .property("icon", icon)
            .property("description", description)
            .build()
    }

    pub fn duration(&self) -> Option<i64> {
        let mut total = 0;
        for task in read_tasks(self.id(), None, None, Some(0)).expect("Failed to read tasks") {
            if let Some(task_duration) = task.duration() {
                total = total + task_duration;
            }
        }
        if total == 0 {
            None
        } else {
            Some(total)
        }
    }

    pub fn id(&self) -> i64 {
        self.property("id")
    }

    pub fn name(&self) -> String {
        self.property("name")
    }

    pub fn archive(&self) -> bool {
        self.property("archive")
    }

    pub fn index(&self) -> i32 {
        self.property("index")
    }

    pub fn icon(&self) -> String {
        self.property("icon")
    }

    pub fn description(&self) -> String {
        self.property("description")
    }
}

impl TryFrom<&Row<'_>> for Project {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Project::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(4)?,
            row.get(5)?,
        ))
    }
}

impl Default for Project {
    fn default() -> Self {
        Project::new(1, String::new(), false, 0, String::new(), String::new())
    }
}
