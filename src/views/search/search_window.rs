use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
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
            move |_controller, keyval, _keycode, _state| {
                let imp = win.imp();

                if let Some(first_child) =
                    imp.search_results.first_child().and_downcast::<gtk::ListBoxRow>() {
                    let step = match keyval {
                        gdk::Key::Up => -1,
                        gdk::Key::Down => 1,
                        _ => return gtk::Inhibit(false)
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
                gtk::Inhibit(false)
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

        if text == imp.prev_search.borrow().as_str() {
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
            imp.search_results.select_row(Some(&first_row));
        } else {
            imp.search_entry.grab_focus();
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
                app_win.activate_action("search.project", None).unwrap();
            }
        } else if let Some(task) = row_imp.task.take() {
            let mut change_project = false;
            if task.project() != 0 {
                let project = read_project(task.project()).unwrap();
                if app_win_project_id != project.id() {
                    app_win.set_property("project", project);
                    change_project = true;
                }
            }

            app_win
                .activate_action(
                    "search.task",
                    Some(&(change_project, task.id()).to_variant()),
                )
                .unwrap();
        }
        self.close();
    }
}
