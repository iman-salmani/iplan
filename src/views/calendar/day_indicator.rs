use gtk::glib;
use gtk::glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar/day_indicator.ui")]
    #[properties(type_wrapper=super::DayIndicator)]
    pub struct DayIndicator {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[template_child]
        pub month: TemplateChild<gtk::Label>,
        #[template_child]
        pub day: TemplateChild<gtk::Label>,
        #[template_child]
        pub weekday: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DayIndicator {
        const NAME: &'static str = "DayIndicator";
        type Type = super::DayIndicator;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                month: TemplateChild::default(),
                day: TemplateChild::default(),
                weekday: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for DayIndicator {
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
    impl WidgetImpl for DayIndicator {}
    impl ButtonImpl for DayIndicator {}
}

glib::wrapper! {
    pub struct DayIndicator(ObjectSubclass<imp::DayIndicator>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Buildable, gtk::Actionable, gtk::Accessible, gtk::ConstraintTarget;
}

impl Default for DayIndicator {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl DayIndicator {
    pub fn new(datetime: glib::DateTime) -> Self {
        let obj = Self::default();
        let imp = obj.imp();
        imp.month.set_label(&datetime.format("%b").unwrap());
        imp.day
            .set_label(datetime.format("%e").unwrap().trim_start());
        imp.weekday.set_label(&datetime.format("%a").unwrap());
        obj.set_datetime(datetime);
        obj
    }
}
