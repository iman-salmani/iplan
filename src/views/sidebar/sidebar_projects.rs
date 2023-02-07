use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/sidebar/sidebar_projects.ui")]
    pub struct SidebarProjects {
        #[template_child]
        pub archive_toggle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub projects_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarProjects {
        const NAME: &'static str = "SidebarProjects";
        type Type = super::SidebarProjects;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SidebarProjects {}
    impl WidgetImpl for SidebarProjects {}
    impl BoxImpl for SidebarProjects {}
}

glib::wrapper! {
    pub struct SidebarProjects(ObjectSubclass<imp::SidebarProjects>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl SidebarProjects {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    #[template_callback]
    fn handle_archive_toggle_button_toggled(&self, _toggle_button: gtk::ToggleButton) {
        self.imp().projects_box.invalidate_filter();
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        println!("New Project");
    }

    #[template_callback]
    fn handle_projects_box_row_activated(
        &self, _projects_box: gtk::ListBox, _project_row: gtk::ListBoxRow) {
        println!("Row activated");
    }
}

