use gettextrs::gettext;
use glib::Properties;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::IPlanApplication;
use crate::db::models::Reminder;
use crate::db::operations::{create_reminder, delete_reminder, update_reminder};
use crate::views::{DateRow, TimeRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/reminder_window.ui")]
    #[properties(wrapper_type=super::ReminderWindow)]
    pub struct ReminderWindow {
        #[property(get, set)]
        pub reminder: RefCell<Reminder>,
        #[property(get, set)]
        pub state: Cell<bool>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub date_row: TemplateChild<DateRow>,
        #[template_child]
        pub time_row: TemplateChild<TimeRow>,
        #[template_child]
        pub delete_group: TemplateChild<adw::PreferencesGroup>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ReminderWindow {
        const NAME: &'static str = "ReminderWindow";
        type Type = super::ReminderWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ReminderWindow {
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
    impl WidgetImpl for ReminderWindow {}
    impl WindowImpl for ReminderWindow {}
}

glib::wrapper! {
    pub struct ReminderWindow(ObjectSubclass<imp::ReminderWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl ReminderWindow {
    pub fn new(
        application: &gtk::Application,
        app_window: &gtk::Window,
        reminder: Reminder,
        state: bool,
    ) -> Self {
        let obj: Self = glib::Object::builder()
            .property("application", application)
            .property("state", state)
            .build();
        obj.set_transient_for(Some(app_window));
        let imp = obj.imp();
        let datetime = reminder.datetime_datetime();
        imp.date_row.set_datetime(&datetime);
        imp.time_row
            .set_time_from_digits(datetime.hour(), datetime.minute(), datetime.seconds());
        if state == true {
            imp.delete_group.set_visible(true);
        }
        obj.set_reminder(reminder);
        obj
    }

    #[template_callback]
    fn handle_cancel_button_clicked(&self, _button: gtk::Button) {
        self.close();
    }

    #[template_callback]
    fn handle_done_button_clicked(&self, _button: gtk::Button) {
        let mut reminder = self.reminder();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        if now > reminder.datetime_duration() {
            let toast = adw::Toast::builder()
                .title(gettext("Time can't be in the past"))
                .build();
            self.imp().toast_overlay.add_toast(toast);
            return;
        }

        if self.state() {
            update_reminder(&reminder).expect("Failed to update record");
        } else {
            reminder = create_reminder(reminder.datetime(), reminder.task(), reminder.priority())
                .expect("Failed to create record");
            self.transient_for()
                .and_downcast::<gtk::Window>()
                .unwrap()
                .activate_action("reminder.created", Some(&reminder.id().to_variant()))
                .expect("Failed to send reminder.created action");
        };

        self.application()
            .and_downcast::<IPlanApplication>()
            .unwrap()
            .send_reminder(reminder);
        self.close();
    }

    #[template_callback]
    fn handle_date_changed(&self, datetime: glib::DateTime, _date_row: DateRow) {
        let imp = self.imp();
        let datetime = datetime.add_seconds(imp.time_row.time() as f64).unwrap();
        let reminder = self.reminder();
        reminder.set_datetime(datetime.to_unix());
    }

    #[template_callback]
    fn handle_time_changed(&self, _time: i32, time_picker: TimeRow) {
        let reminder = self.reminder();
        let prev_datetime = reminder.datetime_datetime();
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
        reminder.set_datetime(datetime.to_unix());
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        delete_reminder(self.reminder().id()).expect("Failed to delete reminder");
        self.close()
    }
}
