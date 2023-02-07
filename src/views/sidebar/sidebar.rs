use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use crate::views::sidebar::SidebarProjects;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/sidebar/sidebar.ui")]
    pub struct Sidebar {}

    #[glib::object_subclass]
    impl ObjectSubclass for Sidebar {
        const NAME: &'static str = "Sidebar";
        type Type = super::Sidebar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            SidebarProjects::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Sidebar {}
    impl WidgetImpl for Sidebar {}
    impl BoxImpl for Sidebar {}
}

glib::wrapper! {
    pub struct Sidebar(ObjectSubclass<imp::Sidebar>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

impl Sidebar {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }
}

