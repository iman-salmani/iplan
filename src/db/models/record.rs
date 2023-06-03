use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::Cell;

mod imp {
    use super::*;
    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Record)]
    pub struct Record {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub start: Cell<i64>,
        #[property(get, set)]
        pub duration: Cell<i64>, // FIXME: Cell<Option<i64>>, because of glib::value::get
        #[property(get, set)]
        pub task: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Record {
        const NAME: &'static str = "Record";
        type Type = super::Record;
    }

    impl ObjectImpl for Record {
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
    pub struct Record(ObjectSubclass<imp::Record>);
}

impl Record {
    pub fn new(id: i64, start: i64, duration: i64, task: i64) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("start", start)
            .property("duration", duration)
            .property("task", task)
            .build()
    }

    pub fn duration_display(duration: i64) -> String {
        if duration == 0 {
            return "0:0".to_string();
        }
        let (min, sec) = (duration / 60, duration % 60);
        if min > 60 {
            let (hour, min) = (min / 60, min % 60);
            format!("{}:{}:{}", hour, min, sec)
        } else {
            format!("{}:{}", min, sec)
        }
    }
}

impl TryFrom<&Row<'_>> for Record {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Record::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
        ))
    }
}

impl Default for Record {
    fn default() -> Self {
        Record::new(1, 0, 0, 1)
    }
}
