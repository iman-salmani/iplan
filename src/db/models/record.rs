use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use rusqlite::{Error, Result, Row};
use std::cell::Cell;
use std::fmt;

mod imp {
    use super::*;

    #[derive(Default, Debug)]
    pub struct Record {
        pub id: Cell<i64>,
        pub start: Cell<i64>,
        pub duration: Cell<i64>, // FIXME: Cell<Option<i64>>, because of glib::value::get
        pub task: Cell<i64>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Record {
        const NAME: &'static str = "Record";
        type Type = super::Record;
    }

    impl ObjectImpl for Record {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::builder("id").build(),
                    glib::ParamSpecInt64::builder("start").build(),
                    glib::ParamSpecInt64::builder("duration").build(),
                    glib::ParamSpecInt64::builder("task").build(),
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
                "start" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.start.set(value);
                }
                "duration" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.duration.set(value);
                }
                "task" => {
                    let value = value.get::<i64>().expect("Value must be a i64");
                    self.task.set(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "id" => self.id.get().to_value(),
                "start" => self.id.get().to_value(),
                "duration" => self.id.get().to_value(),
                "task" => self.id.get().to_value(),
                _ => unimplemented!(),
            }
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

    pub fn id(&self) -> i64 {
        self.property("id")
    }

    pub fn start(&self) -> i64 {
        self.property("start")
    }

    pub fn duration(&self) -> i64 {
        self.property("duration")
    }

    pub fn task(&self) -> i64 {
        self.property("task")
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

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let duration = self.duration();
        if duration != 0 {
            let (min, sec) = (duration / 60, duration % 60);
            if min > 60 {
                let (hour, min) = (duration / 60, duration % 60);
                return write!(f, "{}:{}:{}", hour, min, sec);
            } else {
                return write!(f, "{}:{}", min, sec);
            }
        }
        fmt::Result::Err(fmt::Error)
    }
}
