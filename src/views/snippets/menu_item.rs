use adw::subclass::prelude::*;
use gtk::glib::Properties;
use gtk::{glib, prelude::*};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/snippets/menu_item.ui")]
    #[properties(wrapper_type=super::MenuItem)]
    pub struct MenuItem {
        #[property(get, set, name = "icon-name")]
        pub icon_name: RefCell<String>,
        #[property(get, set)]
        pub label: RefCell<String>,
        #[template_child]
        pub icon_widget: TemplateChild<gtk::Image>,
        #[template_child]
        pub label_widget: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MenuItem {
        const NAME: &'static str = "MenuItem";
        type Type = super::MenuItem;
        type ParentType = gtk::Button;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for MenuItem {
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
    impl WidgetImpl for MenuItem {}
    impl ButtonImpl for MenuItem {}
}

glib::wrapper! {
    pub struct MenuItem(ObjectSubclass<imp::MenuItem>)
        @extends gtk::Widget, gtk::Button,
        @implements gtk::Buildable, gtk::Actionable, gtk::Accessible, gtk::ConstraintTarget;
}

impl Default for MenuItem {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl MenuItem {
    pub fn new() -> Self {
        Self::default()
    }

    fn add_bindings(&self) {
        let imp = self.imp();
        self.bind_property("icon-name", &imp.icon_widget.get(), "icon-name")
            .sync_create()
            .build();

        self.bind_property("label", &imp.label_widget.get(), "label")
            .sync_create()
            .build();
    }
}
