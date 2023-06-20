use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Section)]
    pub struct Section {
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
    impl ObjectSubclass for Section {
        const NAME: &'static str = "Section";
        type Type = super::Section;
    }

    impl ObjectImpl for Section {
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
    pub struct Section(ObjectSubclass<imp::Section>);
}

impl Section {
    pub fn new(id: i64, name: String, project: i64, index: i32) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("name", name)
            .property("project", project)
            .property("index", index)
            .build()
    }
}

impl TryFrom<&Row<'_>> for Section {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Section::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    }
}

impl Default for Section {
    fn default() -> Self {
        Section::new(1, String::new(), 1, 0)
    }
}
