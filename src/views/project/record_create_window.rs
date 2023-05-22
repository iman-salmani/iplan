use glib::once_cell::sync::Lazy;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::models::Record;
use crate::db::operations::create_record;
use crate::views::project::TaskWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/record_create_window.ui")]
    pub struct RecordCreateWindow {
        pub task_id: Cell<i64>,
        pub record: RefCell<Record>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub start_date_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub date_picker_menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub date_picker_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub date_picker: TemplateChild<gtk::Calendar>,
        #[template_child]
        pub start_hour_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub start_minute_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub start_seconds_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_hour_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_minute_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_seconds_spin_button: TemplateChild<gtk::SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecordCreateWindow {
        const NAME: &'static str = "RecordCreateWindow";
        type Type = super::RecordCreateWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecordCreateWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecObject::builder::<Record>("record").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "record" => {
                    let value = value.get::<Record>().expect("value must be a Record");
                    self.record.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "record" => self.record.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for RecordCreateWindow {}
    impl WindowImpl for RecordCreateWindow {}
}

glib::wrapper! {
    pub struct RecordCreateWindow(ObjectSubclass<imp::RecordCreateWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl RecordCreateWindow {
    pub fn new(application: &gtk::Application, app_window: &TaskWindow, task_id: i64) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.task_id.replace(task_id);
        let start = glib::DateTime::now_local().unwrap();
        imp.record
            .replace(Record::new(0, start.to_unix(), 0, task_id));
        imp.start_date_entry.set_text(&start.format("%F").unwrap());
        win
    }

    pub fn record(&self) -> Record {
        self.property("record")
    }

    #[template_callback]
    fn handle_cancel_button_clicked(&self, _button: gtk::Button) {
        self.close();
    }

    #[template_callback]
    fn handle_done_button_clicked(&self, _button: gtk::Button) {
        let record = self.record();
        if record.duration() != 0 {
            let record = create_record(record.start(), record.task(), record.duration())
                .expect("Failed to create record");
            self.transient_for()
                .and_downcast::<TaskWindow>()
                .unwrap()
                .activate_action("record.created", Some(&record.id().to_variant()))
                .expect("Failed to send record.created action");
            self.close();
        } else {
            let toast = adw::Toast::builder().title("duration cannot be 0").build();
            self.imp().toast_overlay.add_toast(&toast);
        }
    }

    #[template_callback]
    fn handle_start_date_entry_changed(&self, entry: adw::EntryRow) {
        let text = entry.text();
        let mut date: Vec<i32> = vec![];
        for num in text.split("-") {
            let num = num.trim();
            if let Ok(num) = num.parse::<i32>() {
                date.push(num);
            } else {
                return;
            }
        }
        if date.len() != 3 {
            return;
        }
        let record = self.record();
        let prev_datetime = glib::DateTime::from_unix_local(record.start()).unwrap();
        if let Ok(datetime) = glib::DateTime::new(
            &glib::TimeZone::local(),
            date[0],
            date[1],
            date[2],
            prev_datetime.hour(),
            prev_datetime.minute(),
            prev_datetime.seconds(),
        ) {
            record.set_property("start", datetime.to_unix());
            self.set_property("record", record);
            self.imp().date_picker.select_day(&datetime);
        }
    }

    #[template_callback]
    fn handle_date_picker_day_selected(&self, calendar: gtk::Calendar) {
        let imp = self.imp();
        if imp.date_picker_popover.is_visible() {
            // prevent edit entry when selected day changes by entry
            imp.date_picker_popover.popdown();
            let date: glib::DateTime = glib::DateTime::new(
                &glib::TimeZone::local(),
                calendar.year(),
                calendar.month() + 1,
                calendar.day(),
                0,
                0,
                0.0,
            )
            .unwrap();
            imp.start_date_entry.set_text(&date.format("%F").unwrap());
        }
    }

    #[template_callback]
    fn handle_start_time_spin_button_value_changed(&self, _button: gtk::SpinButton) {
        let imp = self.imp();
        let record = self.record();
        let prev_datetime = glib::DateTime::from_unix_local(record.start()).unwrap();
        let datetime = glib::DateTime::new(
            &glib::TimeZone::local(),
            prev_datetime.year(),
            prev_datetime.month(),
            prev_datetime.day_of_month(),
            imp.start_hour_spin_button.value_as_int(),
            imp.start_minute_spin_button.value_as_int(),
            imp.start_seconds_spin_button.value_as_int() as f64,
        )
        .unwrap();
        record.set_property("start", datetime.to_unix());
        self.set_property("record", record);
    }

    #[template_callback]
    fn handle_duration_time_spin_button_value_changed(&self, _button: gtk::SpinButton) {
        let imp = self.imp();
        let duration = (imp.duration_hour_spin_button.value_as_int() * 3600)
            + (imp.duration_minute_spin_button.value_as_int() * 60)
            + imp.duration_seconds_spin_button.value_as_int();
        let record = self.record();
        record.set_property("duration", duration as i64);
        self.set_property("record", record);
    }
}
