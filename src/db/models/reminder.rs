use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::Cell;
use std::time::Duration;

mod imp {
    use super::*;
    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type=super::Reminder)]
    pub struct Reminder {
        #[property(get, set)]
        pub id: Cell<i64>,
        #[property(get, set)]
        pub datetime: Cell<i64>,
        #[property(get, set)]
        pub past: Cell<bool>,
        #[property(get, set)]
        pub task: Cell<i64>,
        #[property(get, set)]
        pub priority: Cell<u8>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Reminder {
        const NAME: &'static str = "Reminder";
        type Type = super::Reminder;
    }

    impl ObjectImpl for Reminder {
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
    pub struct Reminder(ObjectSubclass<imp::Reminder>);
}

impl Reminder {
    pub fn new(id: i64, datetime: i64, past: bool, task: i64, priority: u8) -> Self {
        glib::Object::builder()
            .property("id", id)
            .property("datetime", datetime)
            .property("past", past)
            .property("task", task)
            .property("priority", priority)
            .build()
    }

    pub fn datetime_datetime(&self) -> glib::DateTime {
        glib::DateTime::from_unix_local(self.datetime()).unwrap()
    }

    pub fn datetime_duration(&self) -> Duration {
        Duration::from_secs(self.datetime() as u64)
    }
}

impl TryFrom<&Row<'_>> for Reminder {
    type Error = Error;

    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        Ok(Reminder::new(
            row.get(0)?,
            row.get(1)?,
            row.get(2)?,
            row.get(3)?,
            row.get(3)?,
        ))
    }
}

impl Default for Reminder {
    fn default() -> Self {
        Reminder::new(0, 0, false, 0, 2)
    }
}
