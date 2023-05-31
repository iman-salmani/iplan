use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{find_projects, find_tasks, read_project};
use crate::views::search::SearchResult;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/search/search_window.ui")]
    pub struct SearchWindow {
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

    impl ObjectImpl for SearchWindow {}
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
    pub fn new(application: &gtk::Application, app_window: &gtk::Window) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.search_entry.grab_focus();
        let search_entry_controller = gtk::EventControllerKey::new();
        search_entry_controller.connect_key_pressed(glib::clone!(
            @weak win => @default-return gtk::Inhibit(false),
            move |_controller, _keyval, keycode, _state| {
                let imp = win.imp();
                if let Some(first_child) =
                    imp.search_results.first_child().and_downcast::<gtk::ListBoxRow>() {
                    let step = match keycode {
                        111 => -1,  // Up
                        116 => 1,   // Down
                        _ => 0
                    };
                    if step != 0 {
                        if let Some(selected_row) = imp.search_results.selected_row() {
                            if let Some(row) =
                                imp.search_results.row_at_index(selected_row.index() + step) {
                                imp.search_results.select_row(Some(&row));
                            }
                        } else {
                            imp.search_results.select_row(Some(&first_child));
                        }
                    }
                }
                gtk::Inhibit(false)
        }));
        imp.search_entry.add_controller(search_entry_controller);
        win
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

        if text == imp.prev_search.borrow().as_str() {
            return;
        }

        if text.is_empty() {
            let lists = imp.search_results.observe_children();
            for _i in 0..lists.n_items() {
                if let Some(row) = lists.item(0).and_downcast::<gtk::ListBoxRow>() {
                    imp.search_results.remove(&row);
                }
            }
            imp.search_results_placeholder.set_visible(false);
            return;
        } else {
            imp.search_results_placeholder.set_visible(false);
            let archive = imp.show_done_tasks_toggle_button.is_active();
            let projects = find_projects(text, archive).expect("Failed to search projects");
            let tasks = find_tasks(text, archive).expect("Failed to search tasks");
            if imp.search_results.observe_children().n_items()
                == (projects.len() + tasks.len() + 1) as u32
            {
                // One for placeholder
                return;
            }
            let lists = imp.search_results.observe_children();
            for _i in 0..lists.n_items() {
                if let Some(row) = lists.item(0).and_downcast::<gtk::ListBoxRow>() {
                    imp.search_results.remove(&row);
                }
            }
            for project in projects {
                imp.search_results
                    .append(&SearchResult::new(Some(project), None));
            }
            for task in tasks {
                imp.search_results
                    .append(&SearchResult::new(None, Some(task)));
            }
        }

        imp.prev_search.replace(text.to_string());

        if let Some(first_row) = imp
            .search_results
            .first_child()
            .and_downcast::<SearchResult>()
        {
            imp.search_results_placeholder.set_visible(false);
            imp.search_results.select_row(Some(&first_row));
        } else {
            imp.search_entry.grab_focus();
            imp.search_results_placeholder.set_visible(true);
        }
    }

    #[template_callback]
    fn handle_show_done_tasks_toggle_button_toggled(&self, _button: gtk::ToggleButton) {
        let imp = self.imp();
        imp.prev_search.replace(String::new());
        self.handle_search_entry_search_changed(self.imp().search_entry.get());
        imp.search_entry.grab_focus();
    }

    #[template_callback]
    fn handle_search_results_row_activated(&self, row: SearchResult) {
        let row_imp = row.imp();
        let app_win = self.transient_for().unwrap();
        let app_win_project_id = app_win.property::<Project>("project").id();
        if let Some(project) = row_imp.project.take() {
            if app_win_project_id != project.id() {
                app_win.set_property("project", project);
                app_win
                    .activate_action("search.project", None)
                    .expect("Failed to send project.open action");
            }
        } else if let Some(task) = row_imp.task.take() {
            let project = read_project(task.project()).expect("Failed to read project");
            let project_changed = if app_win_project_id != project.id() {
                app_win.set_property("project", project);
                true
            } else {
                false
            };
            if task.done() {
                app_win
                    .activate_action(
                        "search.task-done",
                        Some(&(project_changed, task.id(), task.list()).to_variant()),
                    )
                    .expect("Failed to send project.open action");
            } else {
                app_win
                    .activate_action(
                        "search.task",
                        Some(&(project_changed, task.id()).to_variant()),
                    )
                    .expect("Failed to send project.open action");
            }
        }
        self.close();
    }
}
