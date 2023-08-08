use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{gdk, glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::{Project, Task};
use crate::db::operations::{find_projects, find_tasks};
use crate::views::search::{SearchResult, SearchResultData};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/search/search_window.ui")]
    #[properties(type_wrapper=super::SearchWindow)]
    pub struct SearchWindow {
        #[property(get, set)]
        pub prev_search: RefCell<String>,
        #[template_child]
        pub search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub show_done_tasks_toggle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub search_results: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub search_results_placeholder: TemplateChild<adw::StatusPage>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SearchWindow {
        const NAME: &'static str = "SearchWindow";
        type Type = super::SearchWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SearchWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("project-activated")
                        .param_types([Project::static_type()])
                        .build(),
                    Signal::builder("task-activated")
                        .param_types([Task::static_type()])
                        .build(),
                ]
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

    impl WidgetImpl for SearchWindow {}
    impl WindowImpl for SearchWindow {}
}

glib::wrapper! {
    pub struct SearchWindow(ObjectSubclass<imp::SearchWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl SearchWindow {
    pub fn new<A, W>(application: &A, app_window: &W) -> Self
    where
        A: glib::IsA<gtk::Application>,
        W: glib::IsA<gtk::Window>,
    {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.search_entry.grab_focus();
        let search_entry_controller = gtk::EventControllerKey::new();
        search_entry_controller.connect_key_pressed(glib::clone!(
            @weak win => @default-return glib::Propagation::Proceed,
            move |_controller, keyval, _keycode, _state| {
                let imp = win.imp();

                if let Some(first_child) =
                    imp.search_results.first_child().and_downcast::<gtk::ListBoxRow>() {
                    let step = match keyval {
                        gdk::Key::Up => -1,
                        gdk::Key::Down => 1,
                        _ => return glib::Propagation::Proceed
                    };
                    if let Some(selected_row) = imp.search_results.selected_row() {
                        if let Some(row) =
                            imp.search_results.row_at_index(selected_row.index() + step) {
                            imp.search_results.select_row(Some(&row));
                        }
                    } else {
                        imp.search_results.select_row(Some(&first_child));
                    }

                }
				glib::Propagation::Proceed
        }));
        imp.search_entry.add_controller(search_entry_controller);
        win
    }

    fn clear_search_results(&self) {
        let imp = self.imp();
        let results = imp.search_results.observe_children();
        for _i in 0..results.n_items() {
            if let Some(row) = results.item(0).and_downcast::<gtk::ListBoxRow>() {
                imp.search_results.remove(&row);
            }
        }
    }

    #[template_callback]
    fn handle_search_entry_activate(&self, _entry: gtk::SearchEntry) {
        let imp = self.imp();
        if let Some(selected_row) = imp.search_results.selected_row() {
            self.handle_search_results_row_activated(
                selected_row.downcast::<SearchResult>().unwrap(),
            );
        } else if let Some(first_row) = imp
            .search_results
            .first_child()
            .and_downcast::<SearchResult>()
        {
            self.handle_search_results_row_activated(first_row);
        }
    }

    #[template_callback]
    fn handle_search_entry_search_changed(&self, entry: gtk::SearchEntry) {
        let imp = self.imp();

        let text = entry.text().to_lowercase();
        let text = text.trim();

        if text == self.prev_search() {
            return;
        }

        if text.is_empty() {
            self.clear_search_results();
            imp.search_results_placeholder.set_visible(false);
            return;
        } else {
            let archive = imp.show_done_tasks_toggle_button.is_active();
            let projects = find_projects(text, archive).expect("Failed to search projects");
            let tasks = find_tasks(text, archive).expect("Failed to search tasks");

            imp.search_results_placeholder.set_visible(true);
            if imp.search_results.observe_children().n_items()
                == (projects.len() + tasks.len() + 1) as u32
            {
                // One for placeholder
                // FIXME: check it work with fuzzy match
                return;
            }
            self.clear_search_results();
            for project in projects {
                imp.search_results
                    .append(&SearchResult::new(SearchResultData::Project(project)));
            }
            for task in tasks {
                imp.search_results
                    .append(&SearchResult::new(SearchResultData::Task(task)));
            }
        }

        self.set_prev_search(text.to_string());

        if let Some(first_row) = imp
            .search_results
            .first_child()
            .and_downcast::<SearchResult>()
        {
            imp.search_results.select_row(Some(&first_row));
        } else {
            imp.search_entry.grab_focus();
        }
    }

    #[template_callback]
    fn handle_show_done_tasks_toggle_button_toggled(&self, _button: gtk::ToggleButton) {
        let imp = self.imp();
        self.set_prev_search(String::new());
        self.handle_search_entry_search_changed(self.imp().search_entry.get());
        imp.search_entry.grab_focus();
    }

    #[template_callback]
    fn handle_search_results_row_activated(&self, row: SearchResult) {
        match row.data() {
            SearchResultData::Project(project) => {
                self.emit_by_name::<()>("project-activated", &[&project]);
            }
            SearchResultData::Task(task) => {
                self.emit_by_name::<()>("task-activated", &[&task]);
            }
            SearchResultData::None => unimplemented!(),
        }
        self.close();
    }
}
