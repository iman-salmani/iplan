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
use gtk::{gdk, gio, glib, glib::Properties, prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{create_project, create_section, read_projects};
use crate::views::project::{ProjectEditWindow, ProjectHeader, ProjectLayout, ProjectLists};
use crate::views::{calendar::Calendar, sidebar::SidebarProjects};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/window.ui")]
    #[properties(type_wrapper=super::IPlanWindow)]
    pub struct IPlanWindow {
        #[property(get, set)]
        pub settings: RefCell<Option<gio::Settings>>,
        #[property(get, set)]
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
                    let project = create_project(&gettext("Personal"), "", "")
                        .expect("Failed to create project");
                    create_section(&gettext("Tasks"), project.id()).unwrap();
                    project
                };
                win.set_project(home_project);
                projects_section.select_active_project();
                win.activate_action("project.open", None)
                    .expect("Failed to send project.open action");
            });
            klass.install_action("section.new", None, move |win, _, _| {
                let imp = win.imp();
                imp.project_lists.new_section(win.project().id());
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for IPlanWindow {
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
    impl WidgetImpl for IPlanWindow {}
    impl WindowImpl for IPlanWindow {
        fn close_request(&self) -> glib::signal::Inhibit {
            let obj = self.obj();
            if let Some(settings) = obj.settings() {
                settings
                    .set_int("width", obj.default_width())
                    .expect("failed to set width in settings");
                settings
                    .set_int("height", obj.default_height())
                    .expect("failed to set height in settings");
                settings
                    .set_boolean("is-maximized", obj.is_maximized())
                    .expect("failed to set is-maximized in settings");
                settings
                    .set_boolean("is-fullscreen", obj.is_fullscreened())
                    .expect("failed to set is-fullscreen in settings");
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
                create_project(&gettext("Personal"), "", "").expect("Failed to create project");
            create_section(&gettext("Tasks"), project.id()).unwrap();
            project
        };
        let settings = gio::Settings::new("ir.imansalmani.IPlan.State");
        let obj = glib::Object::builder::<IPlanWindow>()
            .property("application", application)
            .property("project", home_project)
            .property("default-width", settings.int("width"))
            .property("default-height", settings.int("height"))
            .property("maximized", settings.boolean("is-maximized"))
            .property("fullscreened", settings.boolean("is-fullscreen"))
            .build();
        let imp = obj.imp();
        if settings.int("default-project-layout") == 1 {
            imp.project_layout_button
                .set_icon_name("view-columns-symbolic");
            imp.project_lists.set_layout(ProjectLayout::Horizontal);
        } else {
            imp.project_lists.set_layout(ProjectLayout::Vertical);
        }
        obj.set_settings(settings);
        obj.activate_action("project.open", None)
            .expect("Failed to open project");
        imp.sidebar_projects.select_active_project();

        if let Some(display) = gdk::Display::default() {
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/ir/imansalmani/iplan/ui/style.css");
            gtk::style_context_add_provider_for_display(&display, &provider, 400);
        }

        obj
    }

    pub fn change_project(&self, project: Project) {
        let imp = self.imp();
        self.set_project(&project);
        imp.project_header.open_project(&project);
        imp.project_lists.open_project(project.id());
        imp.project_lists.select_task(None);
        imp.sidebar_projects.select_active_project();
        imp.sidebar_projects.check_archive_hidden();
        self.close_calendar();
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
                    imp.project_lists.set_layout(ProjectLayout::Horizontal);
                    self.settings()
                        .unwrap()
                        .set_int("default-project-layout", 1)
                        .expect("Could not set setting.");
                } else {
                    button.set_icon_name("list-symbolic");
                    imp.project_lists.set_layout(ProjectLayout::Vertical);
                    self.settings()
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

        if imp.calendar.is_visible() {
            return;
        }

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
