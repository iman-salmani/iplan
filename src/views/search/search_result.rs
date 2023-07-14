use gettextrs::gettext;
use gtk::{
    glib::{self, PropertySet},
    prelude::*,
    subclass::prelude::*,
};
use std::cell::RefCell;

use crate::db::models::{Project, Task};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum SearchResultData {
    Project(Project),
    Task(Task),
    #[default]
    None,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/search/search_result.ui")]
    pub struct SearchResult {
        pub data: RefCell<SearchResultData>,
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,
        #[template_child]
        pub emoji: TemplateChild<gtk::Label>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub type_label: TemplateChild<gtk::Label>,
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
    pub fn new(data: SearchResultData) -> Self {
        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();
        match data {
            SearchResultData::Project(project) => {
                imp.name.set_label(&project.name());
                imp.emoji.set_visible(true);
                imp.emoji.set_label(&project.icon());
                imp.type_label.set_label(&gettext("project"));
                obj.set_data(SearchResultData::Project(project));
            }
            SearchResultData::Task(task) => {
                imp.name.set_label(&task.name());
                let icon_name = if task.done() {
                    "check-round-outline-whole-symbolic"
                } else {
                    "circle-outline-thick-symbolic"
                };
                imp.icon.set_icon_name(Some(icon_name));
                imp.type_label.set_label(&gettext("task"));
                obj.set_data(SearchResultData::Task(task));
            }
            SearchResultData::None => unimplemented!(),
        }
        obj
    }

    pub fn data(&self) -> SearchResultData {
        self.imp().data.take()
    }

    pub fn set_data(&self, data: SearchResultData) {
        self.imp().data.set(data);
    }
}
