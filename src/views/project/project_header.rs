use adw;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::db::models::Project;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_header.ui")]
    pub struct ProjectHeader {
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub duration_button_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub stat_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectHeader {
        const NAME: &'static str = "ProjectHeader";
        type Type = super::ProjectHeader;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectHeader {}
    impl WidgetImpl for ProjectHeader {}
    impl BoxImpl for ProjectHeader {}
}

glib::wrapper! {
    pub struct ProjectHeader(ObjectSubclass<imp::ProjectHeader>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

impl ProjectHeader {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    // open_project - used by handle_project_open and handle_project_update in window
    pub fn open_project(&self, project: &Project) {
        self.imp().name_label.set_text(&project.name());

        // TODO: Refresh duration
    }
}
