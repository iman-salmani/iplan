use adw::subclass::prelude::*;
use adw::traits::ActionRowExt;
use gtk::glib::{once_cell::sync::Lazy, subclass::*, Properties};
use gtk::{glib, prelude::*};
use std::cell::Cell;

use crate::db::models::Record;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/snippets/time_row.ui")]
    #[properties(wrapper_type=super::TimeRow)]
    pub struct TimeRow {
        #[property(get, set)]
        pub time: Cell<i32>,
        #[template_child]
        pub hour_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub minute_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub seconds_spin_button: TemplateChild<gtk::SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TimeRow {
        const NAME: &'static str = "TimeRow";
        type Type = super::TimeRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TimeRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_bindings();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("time-changed")
                    .param_types([i32::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }
    impl WidgetImpl for TimeRow {}
    impl ListBoxRowImpl for TimeRow {}
    impl PreferencesRowImpl for TimeRow {}
    impl ActionRowImpl for TimeRow {}
}

glib::wrapper! {
    pub struct TimeRow(ObjectSubclass<imp::TimeRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Buildable, gtk::Actionable, gtk::Accessible, gtk::ConstraintTarget;
}

impl Default for TimeRow {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl TimeRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_digits(&self) -> (i32, i32, f64) {
        let time = self.time();
        let (min, sec) = (time / 60, time % 60);
        let (hour, min) = (min / 60, min % 60);
        (hour, min, sec as f64)
    }

    pub fn set_time_from_digits(&self, hour: i32, minute: i32, seconds: f64) {
        self.set_time((hour * 3600) + (minute * 60) + seconds as i32);
    }

    fn add_bindings(&self) {
        let imp = self.imp();
        self.bind_property("time", &imp.hour_spin_button.get(), "value")
            .transform_to(move |binding, _time: i32| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                let (hour, _, _) = obj.get_digits();
                Some(hour as f64)
            })
            .transform_from(|binding, _value: f64| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                Some(obj.calculate_time())
            })
            .bidirectional()
            .sync_create()
            .build();

        self.bind_property("time", &imp.minute_spin_button.get(), "value")
            .transform_to(move |binding, _time: i32| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                let (_, minute, _) = obj.get_digits();
                Some(minute as f64)
            })
            .transform_from(|binding, _value: f64| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                Some(obj.calculate_time())
            })
            .bidirectional()
            .sync_create()
            .build();

        self.bind_property("time", &imp.seconds_spin_button.get(), "value")
            .transform_to(move |binding, _time: i32| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                let (_, _, seconds) = obj.get_digits();
                Some(seconds)
            })
            .transform_from(|binding, _value: f64| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                Some(obj.calculate_time())
            })
            .bidirectional()
            .sync_create()
            .build();

        self.connect_time_notify(|obj| {
            let time = obj.time();
            obj.set_subtitle(&Record::duration_display(time as i64));
            obj.emit_by_name::<()>("time-changed", &[&time]);
        });
    }

    fn calculate_time(&self) -> i32 {
        let imp = self.imp();
        let entries = [
            imp.seconds_spin_button.get(),
            imp.minute_spin_button.get(),
            imp.hour_spin_button.get(),
        ];
        let mut time = 0;
        for (i, entry) in entries.iter().enumerate() {
            entry.remove_css_class("error");
            let value = entry.value_as_int();
            time += value * 60_i32.pow(i as u32);
        }
        if time == 0 {
            imp.seconds_spin_button.add_css_class("error");
        }
        time
    }
}
