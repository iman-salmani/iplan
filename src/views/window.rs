/* window.rs
 *
 * Copyright 2023 Iman Salmani
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gdk, gio, glib, glib::once_cell::sync::Lazy, prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{create_list, create_project, read_list, read_projects};
use crate::views::project::{
    ProjectDoneTasksWindow, ProjectEditWindow, ProjectHeader, ProjectLayout, ProjectLists,
};
use crate::views::sidebar::SidebarProjects;
use crate::views::Calendar;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/window.ui")]
    pub struct IPlanWindow {
        pub settings: RefCell<Option<gio::Settings>>,
        pub project: RefCell<Project>,
        #[template_child]
        pub project_layout_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub sidebar_projects: TemplateChild<SidebarProjects>,
        #[template_child]
        pub project_header: TemplateChild<ProjectHeader>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub project_lists: TemplateChild<ProjectLists>,
        #[template_child]
        pub calendar: TemplateChild<Calendar>,
        #[template_child]
        pub calendar_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IPlanWindow {
        const NAME: &'static str = "IPlanWindow";
        type Type = super::IPlanWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("project.open", None, move |win, _, _| {
                let imp = win.imp();
                let project = win.project();
                imp.project_header.open_project(&project);
                imp.project_lists.open_project(project.id());
                imp.project_lists.select_task(None);
                imp.sidebar_projects.check_archive_hidden();
                win.close_calendar();
            });
            klass.install_action("project.new", None, move |win, _, _| {
                let imp = win.imp();
                let project = win.project();
                imp.project_header.open_project(&project);
                imp.project_lists.open_project(project.id());
                imp.project_header.imp().name_button.emit_clicked();
                imp.sidebar_projects.check_archive_hidden();
            });
            klass.install_action("project.edit", None, move |win, _, _| {
                let window = ProjectEditWindow::new(win.application().unwrap(), win, win.project());
                window.present();
            });
            klass.install_action("project.update", None, move |win, _, _| {
                let imp = win.imp();
                if imp.calendar.is_visible() {
                    return;
                }
                let project = win.project();
                imp.project_header.open_project(&project);
                imp.sidebar_projects.update_project(&project);
            });
            klass.install_action("project.delete", None, move |win, _, _| {
                let projects_section = &win.imp().sidebar_projects;
                projects_section.delete_project(win.project().index());
                let projects = read_projects(true).expect("Failed to read projects");
                let home_project = if let Some(project) = projects.get(0) {
                    project.clone()
                } else {
                    let project =
                        create_project(&gettext("Personal")).expect("Failed to create project");
                    create_list(&gettext("Tasks"), project.id()).expect("Failed to create list");
                    project
                };
                win.set_property("project", home_project);
                projects_section.select_active_project();
                win.activate_action("project.open", None)
                    .expect("Failed to send project.open action");
            });
            klass.install_action("list.new", None, move |win, _, _| {
                let imp = win.imp();
                imp.project_lists.new_list(win.project().id());
            });
            klass.install_action("search.project", None, move |win, _, _| {
                let imp = win.imp();
                let project = win.project();
                imp.project_header.open_project(&project);
                imp.project_lists.open_project(project.id());
                imp.project_lists.select_task(None);
                imp.sidebar_projects.select_active_project();
                imp.sidebar_projects.check_archive_hidden();
                win.close_calendar();
            });
            klass.install_action("search.task", Some("(bx)"), move |win, _, value| {
                let imp = win.imp();
                let (project_changed, task_id) = value.unwrap().get::<(bool, i64)>().unwrap();
                if project_changed {
                    let project = win.project();
                    imp.project_header.open_project(&project);
                    imp.project_lists.open_project(project.id());
                    imp.sidebar_projects.select_active_project();
                    imp.sidebar_projects.check_archive_hidden();
                }
                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                glib::idle_add_once(move || {
                    tx.send("").expect("Something happens");
                });
                rx.attach(
                    None,
                    glib::clone!(
                            @weak imp => @default-return glib::Continue(false),
                            move |_text| {
                                imp.project_lists.select_task(Some(task_id));
                                glib::Continue(false)
                            }
                    ),
                );
                win.close_calendar();
            });
            klass.install_action("search.task-done", Some("(bxx)"), move |win, _, value| {
                let imp = win.imp();
                let (project_changed, task_id, list_id) =
                    value.unwrap().get::<(bool, i64, i64)>().unwrap();
                if project_changed {
                    let project = win.project();
                    imp.project_header.open_project(&project);
                    imp.project_lists.open_project(project.id());
                    imp.sidebar_projects.select_active_project();
                    imp.sidebar_projects.check_archive_hidden();
                }
                let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                glib::idle_add_once(move || {
                    tx.send("").expect("Something happens");
                });
                let list = read_list(list_id).expect("Failed to read list");
                let window = ProjectDoneTasksWindow::new(win.application().unwrap(), win, list);
                window.present();
                rx.attach(
                    None,
                    glib::clone!(
                            @weak window => @default-return glib::Continue(false),
                            move |_text| {
                                window.select_task(task_id);
                                glib::Continue(false)
                            }
                    ),
                );
                win.close_calendar();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IPlanWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecObject::builder::<Project>("project").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "project" => {
                    let value = value.get::<Project>().expect("value must be a Project");
                    self.project.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "project" => self.project.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for IPlanWindow {}
    impl WindowImpl for IPlanWindow {
        fn close_request(&self) -> glib::signal::Inhibit {
            if let Some(settings) = self.settings.borrow().as_ref() {
                let obj = self.obj();
                settings
                    .set_int("width", obj.property("default-width"))
                    .expect("failed to set width in settings");
                settings
                    .set_int("height", obj.property("default-height"))
                    .expect("failed to set height in settings");
                settings
                    .set_boolean("is-maximized", obj.property("maximized"))
                    .expect("failed to set width in settings");
                settings
                    .set_boolean("is-fullscreen", obj.property("fullscreened"))
                    .expect("failed to set width in settings");
            }
            self.parent_close_request()
        }
    }
    impl ApplicationWindowImpl for IPlanWindow {}
    impl AdwApplicationWindowImpl for IPlanWindow {}
}

glib::wrapper! {
    pub struct IPlanWindow(ObjectSubclass<imp::IPlanWindow>)
        @extends gtk::Widget, gtk::Root, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

#[gtk::template_callbacks]
impl IPlanWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        let projects = read_projects(true).expect("Failed to read projects");
        let home_project = if let Some(project) = projects.get(0) {
            project.clone()
        } else {
            let project: Project =
                create_project(&gettext("Personal")).expect("Failed to create project");
            create_list(&gettext("Tasks"), project.id()).expect("Failed to create list");
            project
        };
        let settings = gio::Settings::new("ir.imansalmani.IPlan.State");
        let window = glib::Object::builder::<IPlanWindow>()
            .property("application", application)
            .property("project", home_project)
            .property("default-width", settings.int("width"))
            .property("default-height", settings.int("height"))
            .property("maximized", settings.boolean("is-maximized"))
            .property("fullscreened", settings.boolean("is-fullscreen"))
            .build();
        let imp = window.imp();
        if settings.int("default-project-layout") == 1 {
            imp.project_layout_button
                .set_icon_name("view-columns-symbolic");
            imp.project_lists
                .set_layout(&window, ProjectLayout::Horizontal);
        }
        imp.settings.replace(Some(settings));
        window
            .activate_action("project.open", None)
            .expect("Failed to open project");
        imp.sidebar_projects.select_active_project();
        imp.project_lists.open_project(window.project().id());

        if let Some(display) = gdk::Display::default() {
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/ir/imansalmani/iplan/ui/style.css");
            gtk::style_context_add_provider_for_display(&display, &provider, 400)
        }

        window
    }

    pub fn project(&self) -> Project {
        self.property("project")
    }

    fn close_calendar(&self) {
        let imp = self.imp();
        imp.calendar.set_visible(false);
        imp.calendar_button.add_css_class("flat");
    }

    #[template_callback]
    fn handle_project_layout_button_clicked(&self, button: gtk::Button) {
        let imp = self.imp();
        match button.icon_name() {
            Some(icon_name) => {
                if icon_name == "list-symbolic" {
                    button.set_icon_name("view-columns-symbolic");
                    imp.project_lists
                        .set_layout(self, ProjectLayout::Horizontal);
                    imp.settings
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .set_int("default-project-layout", 1)
                        .expect("Could not set setting.");
                } else {
                    button.set_icon_name("list-symbolic");
                    imp.project_lists.set_layout(self, ProjectLayout::Vertical);
                    imp.settings
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .set_int("default-project-layout", 0)
                        .expect("Could not set setting.");
                }
                imp.project_lists.open_project(self.project().id());
            }
            None => button.set_icon_name("list-symbolic"),
        }
    }

    #[template_callback]
    fn handle_calendar_button_clicked(&self, button: gtk::Button) {
        let imp = self.imp();
        button.remove_css_class("flat");
        imp.project.take();
        imp.calendar.set_visible(true);
        if imp.calendar.imp().stack.visible_child_name().is_none() {
            imp.calendar.open_today();
        } else {
            imp.calendar.refresh();
        }
        let projects_box: &gtk::ListBox = imp.sidebar_projects.imp().projects_box.as_ref();
        projects_box.unselect_row(&projects_box.selected_row().unwrap());
    }

    #[template_callback]
    fn handle_calendar_today_clicked(&self, _: gtk::Button) {
        let imp = self.imp();
        imp.calendar.open_today();
    }
}
