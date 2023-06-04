use adw;
use adw::subclass::prelude::*;
use adw::traits::{ExpanderRowExt, PreferencesRowExt};
use gettextrs::gettext;
use gtk::{glib, glib::Properties, prelude::*};
use std::cell::RefCell;

use crate::db::models::Record;
use crate::db::operations::{delete_record, update_record};
use crate::views::{project::TaskWindow, DateRow, TimeRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/record_row.ui")]
    #[properties(wrapper_type=super::RecordRow)]
    pub struct RecordRow {
        #[property(get, set)]
        pub record: RefCell<Record>,
        #[template_child]
        pub start_date_row: TemplateChild<DateRow>,
        #[template_child]
        pub start_time_row: TemplateChild<TimeRow>,
        #[template_child]
        pub duration_row: TemplateChild<TimeRow>,
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
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
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
        let start = glib::DateTime::from_unix_local(record.start())
            .expect("Failed to create glib::DateTime from Record::start");
        let obj: Self = glib::Object::builder().property("record", record).build();
        let imp = obj.imp();
        obj.set_labels();
        imp.start_date_row.set_datetime(&start);
        imp.start_time_row
            .set_time_from_digits(start.hour(), start.minute(), start.seconds());
        imp.duration_row.set_time(duration as i32);
        obj
    }

    fn set_labels(&self) {
        let record = self.record();
        let start = glib::DateTime::from_unix_local(record.start())
            .expect("Failed to create glib::DateTime from Record::start");
        let duration = record.duration();

        self.set_title(&Record::duration_display(duration));

        let start_date_text = start.format("%B %e").unwrap();
        let end = start.add_seconds(duration as f64).unwrap();
        let mut end_date_text = end.format("%B %e").unwrap().to_string();
        end_date_text = if start_date_text == end_date_text {
            String::new()
        } else {
            format!("{end_date_text}, ")
        };
        self.set_subtitle(&format!(
            "{}, {} {} {}{}",
            start_date_text,
            start.format("%H:%M").unwrap(),
            gettext("until"),
            end_date_text,
            end.format("%H:%M").unwrap()
        ));
    }

    fn refresh(&self) {
        self.set_labels();
        if self.parent().is_some() {
            self.activate_action("task.duration-update", None)
                .expect("Failed to send task.duration-update action");
        }
    }

    #[template_callback]
    fn handle_start_date_changed(&self, datetime: glib::DateTime, _date_row: DateRow) {
        let imp = self.imp();
        let datetime = datetime
            .add_seconds(imp.start_time_row.time() as f64)
            .unwrap();
        let record = self.record();
        record.set_start(datetime.to_unix());
        self.set_labels();
        update_record(&record).expect("Failed to update record");
    }

    #[template_callback]
    fn handle_start_time_changed(&self, _time: i32, time_picker: TimeRow) {
        let record = self.record();
        let prev_datetime = glib::DateTime::from_unix_local(record.start()).unwrap();
        let (hour, min, sec) = time_picker.get_digits();
        let datetime = glib::DateTime::new(
            &glib::TimeZone::local(),
            prev_datetime.year(),
            prev_datetime.month(),
            prev_datetime.day_of_month(),
            hour,
            min,
            sec,
        )
        .unwrap();
        record.set_start(datetime.to_unix());
        update_record(&record).expect("Failed to update record");
        self.refresh();
    }

    #[template_callback]
    fn handle_duration_time_changed(&self, time: i32, _: TimeRow) {
        if time == 0 {
            let toast = adw::Toast::builder()
                .title(gettext("Duration can't be 0"))
                .build();
            self.root()
                .and_downcast::<TaskWindow>()
                .unwrap()
                .imp()
                .toast_overlay
                .add_toast(toast);
            return;
        }

        let record = self.record();
        record.set_duration(time as i64);
        update_record(&record).expect("Failed to update record");
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
