use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::{Project, Task};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/search/search_result.ui")]
    pub struct SearchResult {
        pub project: RefCell<Option<Project>>,
        pub task: RefCell<Option<Task>>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub type_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub done_check_button: TemplateChild<gtk::CheckButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchResult {
        const NAME: &'static str = "SearchResult";
        type Type = super::SearchResult;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchResult {}
    impl WidgetImpl for SearchResult {}
    impl ListBoxRowImpl for SearchResult {}
}

glib::wrapper! {
    pub struct SearchResult(ObjectSubclass<imp::SearchResult>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Buildable;
}

impl SearchResult {
    pub fn new(project: Option<Project>, task: Option<Task>) -> Self {
        let win: Self = glib::Object::builder().build();
        let imp = win.imp();
        if let Some(project) = project {
            imp.name_label.set_label(&project.name());
            imp.type_label.set_label("Project");
            imp.done_check_button.set_visible(false);
            imp.project.replace(Some(project));
        } else if let Some(task) = task {
            imp.name_label.set_label(&task.name());
            imp.type_label.set_label("Project");
            imp.done_check_button.set_active(task.done());
            imp.task.replace(Some(task));
        }
        win
    }
}
