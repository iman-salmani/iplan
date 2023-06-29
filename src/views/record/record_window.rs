use gettextrs::gettext;
use glib::Properties;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::models::Record;
use crate::db::operations::{create_record, delete_record, update_record};
use crate::views::snippets::{DateRow, TimeRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/record/record_window.ui")]
    #[properties(wrapper_type=super::RecordWindow)]
    pub struct RecordWindow {
        #[property(get, set)]
        pub record: RefCell<Record>,
        #[property(get, set)]
        pub state: Cell<bool>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub start_date_row: TemplateChild<DateRow>,
        #[template_child]
        pub start_time_row: TemplateChild<TimeRow>,
        #[template_child]
        pub end_date_row: TemplateChild<DateRow>,
        #[template_child]
        pub end_time_row: TemplateChild<TimeRow>,
        #[property(get, set)]
        pub end_datetime: RefCell<i64>,
        #[template_child]
        pub duration_row: TemplateChild<TimeRow>,
        #[template_child]
        pub delete_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecordWindow {
        const NAME: &'static str = "RecordWindow";
        type Type = super::RecordWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecordWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_bindings();
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
    impl WidgetImpl for RecordWindow {}
    impl WindowImpl for RecordWindow {}
}

glib::wrapper! {
    pub struct RecordWindow(ObjectSubclass<imp::RecordWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl RecordWindow {
    pub fn new(
        application: &gtk::Application,
        app_window: &gtk::Window,
        record: Record,
        state: bool,
    ) -> Self {
        let obj: Self = glib::Object::builder()
            .property("application", application)
            .property("state", state)
            .build();
        obj.set_transient_for(Some(app_window));
        let imp = obj.imp();
        let start_unix = record.start();
        let start = glib::DateTime::from_unix_local(start_unix).unwrap();
        let duration = record.duration();
        obj.set_record(record);
        imp.start_date_row.set_datetime(&start);
        imp.start_time_row
            .set_time_from_digits(start.hour(), start.minute(), start.seconds());
        imp.duration_row.set_time(duration as i32);
        if state {
            imp.delete_group.set_visible(true);
        }
        obj
    }

    fn add_bindings(&self) {
        let imp = self.imp();

        imp.duration_row
            .bind_property::<Self>("time", self, "end_datetime")
            .transform_to(|binding, time: i32| {
                let obj = binding.target().and_downcast::<Self>().unwrap();
                let imp = obj.imp();
                let unix = obj.record().start() + time as i64;
                imp.end_date_row.set_datetime_from_unix(unix);
                Some(unix)
            })
            .transform_from(|binding, end_datetime: i64| {
                let obj = binding.target().and_downcast::<Self>().unwrap();
                let duration = end_datetime - obj.record().start();
                if duration.is_negative() {
                    Some(0)
                } else {
                    Some(duration as i32)
                }
            })
            .sync_create()
            .bidirectional()
            .build();
        self.bind_property::<TimeRow>("end_datetime", &imp.end_time_row, "time")
            .transform_to(|_binding, end_datetime: i64| {
                let datetime = glib::DateTime::from_unix_local(end_datetime).unwrap();
                let time =
                    (datetime.hour() * 3600) + (datetime.minute() * 60) + datetime.seconds() as i32;
                Some(time)
            })
            .transform_from(|binding, time: i32| {
                let obj = binding.source().and_downcast::<Self>().unwrap();
                let datetime = obj
                    .imp()
                    .end_date_row
                    .date()
                    .add_seconds(time as f64)
                    .unwrap();
                Some(datetime.to_unix())
            })
            .sync_create()
            .bidirectional()
            .build();

        self.connect_end_datetime_notify(|obj| {
            let imp = obj.imp();
            let record = obj.record();
            let difference = obj.end_datetime() - record.start();
            if difference > 0 {
                imp.end_date_row.remove_css_class("error");
                imp.end_time_row.remove_css_class("error");
            } else if difference > 24 * 60 * -60 {
                imp.end_date_row.remove_css_class("error");
                imp.end_time_row.add_css_class("error");
            } else {
                imp.end_date_row.add_css_class("error");
                imp.end_time_row.remove_css_class("error");
            }
        });
    }

    fn set_duration(&self, duration: i64) {
        let imp = self.imp();
        if duration.is_negative() {
            imp.duration_row.set_time(0);
        } else {
            imp.duration_row.set_time(duration as i32);
        };
    }

    #[template_callback]
    fn handle_cancel_button_clicked(&self, _button: gtk::Button) {
        self.close();
    }

    #[template_callback]
    fn handle_done_button_clicked(&self, _button: gtk::Button) {
        let record = self.record();

        if record.duration() <= 0 {
            let toast = adw::Toast::builder()
                .title(gettext("Duration can't be 0"))
                .build();
            self.imp().toast_overlay.add_toast(toast);
            return;
        }

        if self.end_datetime() > glib::DateTime::now_local().unwrap().to_unix() {
            let toast = adw::Toast::builder()
                .title(gettext("Record end time can't be in future"))
                .build();
            self.imp().toast_overlay.add_toast(toast);
            return;
        }

        if self.state() {
            update_record(&record).expect("Failed to update record");
        } else {
            let record = create_record(record.start(), record.task(), record.duration())
                .expect("Failed to create record");
            self.transient_for()
                .and_downcast::<gtk::Window>()
                .unwrap()
                .activate_action("record.created", Some(&record.id().to_variant()))
                .expect("Failed to send record.created action");
        }

        self.close();
    }

    #[template_callback]
    fn handle_start_date_changed(&self, datetime: glib::DateTime, _date_row: DateRow) {
        let imp = self.imp();
        let datetime = datetime
            .add_seconds(imp.start_time_row.time() as f64)
            .unwrap()
            .to_unix();
        self.record().set_start(datetime);
        self.set_duration(self.end_datetime() - datetime);
    }

    #[template_callback]
    fn handle_start_time_changed(&self, _time: i32, time_row: TimeRow) {
        let record = self.record();
        let prev_datetime = glib::DateTime::from_unix_local(record.start()).unwrap();
        let (hour, min, sec) = time_row.get_digits();
        let datetime = glib::DateTime::new(
            &glib::TimeZone::local(),
            prev_datetime.year(),
            prev_datetime.month(),
            prev_datetime.day_of_month(),
            hour,
            min,
            sec,
        )
        .unwrap()
        .to_unix();
        record.set_start(datetime);
        self.set_duration(self.end_datetime() - datetime);
    }

    #[template_callback]
    fn handle_end_date_changed(&self, datetime: glib::DateTime, _: DateRow) {
        let datetime = datetime
            .add_seconds(self.imp().end_time_row.time() as f64)
            .unwrap()
            .to_unix();
        self.set_end_datetime(datetime);
    }

    #[template_callback]
    fn handle_duration_time_changed(&self, time: i32, _: TimeRow) {
        let record = self.record();
        record.set_duration(time as i64);
    }

    #[template_callback]
    fn handle_delete_activated(&self, _: adw::ActionRow) {
        delete_record(self.record().id()).expect("Failed to delete record");
        self.close();
    }
}
