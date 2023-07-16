use gettextrs::gettext;
use gtk::{
    glib,
    glib::{FromVariant, Properties},
    prelude::*,
    subclass::prelude::*,
};
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
            return "0:00".to_string();
        }
        let (min, sec) = (duration / 60, duration % 60);
        if min < 60 {
            format!("{}:{:0>2}", min, sec)
        } else {
            let (hour, min) = (min / 60, min % 60);
            if hour < 24 {
                format!("{}:{:0>2}:{:0>2}", hour, min, sec)
            } else {
                let (day, hour) = (hour / 24, hour % 24);
                let day_label = if day == 1 {
                    gettext("day")
                } else {
                    gettext("days")
                };
                format!("{} {} {:0>2}:{:0>2}", day, day_label, hour, min)
            }
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
        Record::new(0, 0, 0, 1)
    }
}

impl StaticVariantType for Record {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::from(glib::VariantTy::new("(xxxx)").unwrap())
    }
}

impl ToVariant for Record {
    fn to_variant(&self) -> glib::Variant {
        glib::Variant::from((self.id(), self.start(), self.duration(), self.task()))
    }
}

impl FromVariant for Record {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let (id, start, duration, task): (i64, i64, i64, i64) = variant.get()?;
        Some(Record::new(id, start, duration, task))
    }
}
