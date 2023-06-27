use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/snippets/chart_bar.ui")]
    pub struct ChartBar {
        #[template_child]
        pub level: TemplateChild<gtk::LevelBar>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ChartBar {
        const NAME: &'static str = "ChartBar";
        type Type = super::ChartBar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ChartBar {}
    impl WidgetImpl for ChartBar {}
    impl BoxImpl for ChartBar {}
}

glib::wrapper! {
    pub struct ChartBar(ObjectSubclass<imp::ChartBar>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable, gtk::Orientable, gtk::Accessible, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl ChartBar {
    pub fn new(level: f64, tooltip: &str) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();
        imp.level.set_value(level);
        imp.level.set_tooltip_text(Some(tooltip));
        obj
    }
}
