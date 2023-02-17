use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::db::operations::read_lists;
use super::ProjectList;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_lists.ui")]
    pub struct ProjectLists {
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub lists_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectLists {
        const NAME: &'static str = "ProjectLists";
        type Type = super::ProjectLists;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectLists {
        fn dispose(&self) {
            self.obj().first_child().unwrap().unparent();
        }
    }
    impl BuildableImpl for ProjectLists {}
    impl WidgetImpl for ProjectLists {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.parent_request_mode();
            gtk::SizeRequestMode::HeightForWidth
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.obj()
                .first_child()
                .unwrap()
                .measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.obj()
                .first_child()
                .unwrap()
                .size_allocate(&gtk::Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct ProjectLists(ObjectSubclass<imp::ProjectLists>)
        @extends gtk::Widget,
        @implements gtk::Buildable;
}

impl ProjectLists {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn open_project(&self, project_id: i64) {
        let imp = self.imp();

        loop {
            if let Some(child) = imp.lists_box.first_child() {
                imp.lists_box.remove(&child);
            } else {
                break
            }
        }

        for list in read_lists(project_id).expect("Failed to read lists") {
            let project_list = ProjectList::new(list);
            imp.lists_box.append(&project_list);
            project_list.init_widgets(project_id);
        }

        if imp.lists_box.first_child().is_none() {
            imp.lists_box.append(&imp.placeholder.get());
        }

        // TODO: Set layout

        // TODO: Select target task
    }
}

