use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct List {
        pub id: Cell<i64>,
        pub name: RefCell<String>,
        pub project: Cell<i64>,
        pub index: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for List {
        const NAME: &'static str = "List";
        type Type = super::List;
    }

    impl ObjectImpl for List {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").build(),
                    glib::ParamSpecString::builder("name").build(),
                    glib::ParamSpecInt64::builder("project").build(),
                    glib::ParamSpecInt::builder("index").build(),
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
                "project" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.project.set(value);
                }
                "index" => {
                    let value = value.get::<i32>().expect("Value must be a i32");
                    self.index.set(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "name" => self.name.borrow().to_string().to_value(),
                "project" => self.project.get().to_value(),
                "index" => self.index.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct List(ObjectSubclass<imp::List>);
}

impl List {
    pub fn new(id: i64, name: String, project: i64, index: i32) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("project", project)
            .property("index", index)
            .build()
    }

    pub fn id(&self) -> i64 {
        self.property("id")
    }

    pub fn name(&self) -> String {
        self.property("name")
    }

    pub fn project(&self) -> i64 {
        self.property("project")
    }

    pub fn index(&self) -> i32 {
        self.property("index")
    }
}

impl TryFrom<&Row<'_>> for List {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(List::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    }
}

impl Default for List {
    fn default() -> Self {
        List::new(1, String::new(), 1, 0)
    }
}

