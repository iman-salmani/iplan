use adw;
use adw::subclass::prelude::*;
use adw::traits::{ExpanderRowExt, PreferencesRowExt};
use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*};
use std::cell::RefCell;

use crate::db::models::Record;
use crate::db::operations::{delete_record, update_record};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/record_row.ui")]
    pub struct RecordRow {
        pub record: RefCell<Record>,
        #[template_child]
        pub start_row: TemplateChild<adw::ExpanderRow>,
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
        pub duration_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub duration_hour_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_minute_spin_button: TemplateChild<gtk::SpinButton>,
        #[template_child]
        pub duration_seconds_spin_button: TemplateChild<gtk::SpinButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecordRow {
        const NAME: &'static str = "RecordRow";
        type Type = super::RecordRow;
        type ParentType = adw::ExpanderRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecordRow {
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
    impl WidgetImpl for RecordRow {}
    impl ListBoxRowImpl for RecordRow {}
    impl PreferencesRowImpl for RecordRow {}
    impl ExpanderRowImpl for RecordRow {}
}

glib::wrapper! {
    pub struct RecordRow(ObjectSubclass<imp::RecordRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ExpanderRow,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl RecordRow {
    pub fn new(record: Record) -> Self {
        let duration = record.duration();
        let duration_text = Record::duration_display(duration);
        let start = glib::DateTime::from_unix_local(record.start())
            .expect("Failed to create glib::DateTime from Record::start");
        let row: Self = glib::Object::builder().property("record", record).build();
        let imp = row.imp();
        row.set_title(&duration_text);
        imp.duration_row.set_subtitle(&duration_text);
        let (min, sec) = (duration / 60, duration % 60);
        let (hour, min) = (min / 60, min % 60);
        imp.duration_hour_spin_button.set_value(hour as f64);
        imp.duration_minute_spin_button.set_value(min as f64);
        imp.duration_seconds_spin_button.set_value(sec as f64);
        row.set_subtitle(&format!(
            "{} > {}",
            start.format("%B %e - %H:%M").unwrap(),
            start
                .add_seconds(duration as f64)
                .unwrap()
                .format("%H:%M")
                .unwrap()
        ));
        imp.start_row
            .set_subtitle(&start.format("%F %H:%M:%S").unwrap());
        imp.start_date_entry.set_text(&start.format("%F").unwrap());
        imp.start_hour_spin_button.set_value(start.hour() as f64);
        imp.start_minute_spin_button
            .set_value(start.minute() as f64);
        imp.start_seconds_spin_button
            .set_value(start.seconds() as f64);
        row
    }

    pub fn record(&self) -> Record {
        self.property("record")
    }

    fn refresh(&self) {
        let imp = self.imp();
        let record = self.record();
        let start = glib::DateTime::from_unix_local(record.start())
            .expect("Failed to create glib::DateTime from Record::start");
        let duration = record.duration();
        let duration_text = Record::duration_display(duration);
        self.set_title(&duration_text);
        imp.duration_row.set_subtitle(&duration_text);
        self.set_subtitle(&format!(
            "{} > {}",
            start.format("%B %e - %H:%M").unwrap(),
            start
                .add_seconds(duration as f64)
                .unwrap()
                .format("%H:%M")
                .unwrap()
        ));
        imp.start_row
            .set_subtitle(&start.format("%F %H:%M:%S").unwrap());
        if self.parent().is_some() {
            self.activate_action("task.duration-update", None)
                .expect("Failed to send task.duration-update action");
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
            update_record(&record).expect("Failed to update record");
            self.set_property("record", record);
            self.refresh();
        }
    }

    #[template_callback]
    fn handle_date_picker_popover_show(&self, _popover: gtk::Popover) {
        let record = self.record();
        let datetime = glib::DateTime::from_unix_local(record.start()).unwrap();
        self.imp().date_picker.select_day(&datetime);
    }

    #[template_callback]
    fn handle_date_picker_day_selected(&self, calendar: gtk::Calendar) {
        calendar.year();
        let imp = self.imp();
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
        update_record(&record).expect("Failed to update record");
        self.set_property("record", record);
        self.refresh();
    }

    #[template_callback]
    fn handle_duration_time_spin_button_value_changed(&self, _button: gtk::SpinButton) {
        let imp = self.imp();
        let duration = (imp.duration_hour_spin_button.value_as_int() * 3600)
            + (imp.duration_minute_spin_button.value_as_int() * 60)
            + imp.duration_seconds_spin_button.value_as_int();
        let record = self.record();
        record.set_property("duration", duration as i64);
        update_record(&record).expect("Failed to update record");
        self.set_property("record", record);
        self.refresh();
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        delete_record(self.record().id()).expect("Failed to delete record");
        self.activate_action("record.delete", None)
            .expect("Failed to send record.delete action");
        let records_box = self.parent().and_downcast::<gtk::ListBox>().unwrap();
        records_box.remove(self);
    }
}
