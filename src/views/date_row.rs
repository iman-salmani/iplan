use adw::subclass::prelude::*;
use adw::traits::ActionRowExt;
use gtk::glib::{once_cell::sync::Lazy, subclass::*};
use gtk::{glib, prelude::*};

const DATE_FORMAT: &str = "%B %e, %Y";

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/date_row.ui")]
    pub struct DateRow {
        #[template_child]
        pub calendar: TemplateChild<gtk::Calendar>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DateRow {
        const NAME: &'static str = "DateRow";
        type Type = super::DateRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for DateRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("date-changed")
                    .param_types([glib::DateTime::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for DateRow {}
    impl ListBoxRowImpl for DateRow {}
    impl PreferencesRowImpl for DateRow {}
    impl ActionRowImpl for DateRow {}
}

glib::wrapper! {
    pub struct DateRow(ObjectSubclass<imp::DateRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Buildable, gtk::Actionable, gtk::Accessible, gtk::ConstraintTarget;
}

impl Default for DateRow {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl DateRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_date(&self, year: u16, month: u8, day: u8) {
        let imp = self.imp();
        imp.calendar.set_year(year as i32);
        imp.calendar.set_month(month as i32 - 1);
        imp.calendar.set_day(day as i32);
        let datetime = self.calculate_datetime();
        self.set_subtitle(&datetime.format(DATE_FORMAT).unwrap());
    }

    pub fn calculate_datetime(&self) -> glib::DateTime {
        let calendar: &gtk::Calendar = self.imp().calendar.as_ref();
        glib::DateTime::new(
            &glib::TimeZone::local(),
            calendar.year(),
            calendar.month() + 1,
            calendar.day(),
            0,
            0,
            0.0,
        )
        .unwrap()
    }

    #[template_callback]
    fn handle_day_selected(&self, _calendar: gtk::Calendar) {
        self.imp().menu_button.popdown();
        let datetime = self.calculate_datetime();
        self.set_subtitle(&datetime.format(DATE_FORMAT).unwrap());
        self.emit_by_name::<()>("date-changed", &[&datetime]);
    }
}
