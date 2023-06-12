use adw::subclass::prelude::*;
use adw::traits::ActionRowExt;
use gettextrs::gettext;
use gtk::glib::{once_cell::sync::Lazy, subclass::*, Properties};
use gtk::{glib, prelude::*};
use std::cell::Cell;
const DATE_FORMAT: &str = "%B %e, %Y";

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/date_row.ui")]
    #[properties(type_wrapper=super::DateRow)]
    pub struct DateRow {
        #[template_child]
        pub calendar: TemplateChild<gtk::Calendar>,
        #[template_child]
        pub menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub clear_button: TemplateChild<gtk::Button>,
        #[property(get, set)]
        pub clear_option: Cell<bool>,
        #[property(get, set)]
        pub skip: Cell<bool>,
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

    pub fn set_datetime(&self, datetime: &glib::DateTime) {
        let imp = self.imp();
        self.set_skip(true);
        imp.calendar.select_day(datetime);
        self.set_skip(false);
        self.refresh_row(datetime);
        self.show_clear_button();
    }

    pub fn set_datetime_from_unix(&self, unix: i64) {
        let datetime = glib::DateTime::from_unix_local(unix).unwrap();
        self.set_datetime(&datetime);
    }

    pub fn date(&self) -> glib::DateTime {
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

    fn refresh_row(&self, datetime: &glib::DateTime) {
        let now = glib::DateTime::now_local().unwrap().ymd();
        let subtitle = if now == datetime.ymd() {
            gettext("Today")
        } else {
            datetime.format(DATE_FORMAT).unwrap().to_string()
        };
        self.set_subtitle(&subtitle);
    }

    fn show_clear_button(&self) {
        if self.clear_option() {
            self.imp().clear_button.set_visible(true);
        }
    }

    #[template_callback]
    fn handle_clear_clicked(&self, clear_button: gtk::Button) {
        let imp = self.imp();
        clear_button.set_visible(false);
        self.set_subtitle("");
        imp.menu_button.popdown();
        self.emit_by_name::<()>(
            "date-changed",
            &[&glib::DateTime::from_unix_local(0).unwrap()],
        );
    }

    #[template_callback]
    fn handle_today_clicked(&self, _: gtk::Button) {
        let imp = self.imp();
        let now = glib::DateTime::now_local().unwrap();
        imp.calendar.select_day(&now);
        self.refresh_row(&now);
        self.show_clear_button();
    }

    #[template_callback]
    fn handle_day_selected(&self, _calendar: gtk::Calendar) {
        if self.skip() {
            return;
        }

        let imp = self.imp();
        imp.menu_button.popdown();
        self.show_clear_button();
        let datetime = self.date();
        self.refresh_row(&datetime);
        self.emit_by_name::<()>("date-changed", &[&datetime]);
    }
}
