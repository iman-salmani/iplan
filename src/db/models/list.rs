use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::List)]
    pub struct List {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub name: RefCell<String>,
        #[property(get, set)]
        pub project: Cell<i64>,
        #[property(get, set)]
        pub index: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for List {
        const NAME: &'static str = "List";
        type Type = super::List;
    }

    impl ObjectImpl for List {
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
