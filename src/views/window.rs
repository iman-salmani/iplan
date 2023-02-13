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
use gtk::{gio, glib, glib::once_cell::sync::Lazy, prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{create_list, create_project, read_projects};
use crate::views::project::ProjectHeader;
use crate::views::sidebar::Sidebar;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/window.ui")]
    pub struct IPlanWindow {
        pub settings: gio::Settings,
        pub project: RefCell<Project>,
        #[template_child]
        pub project_layout_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub project_header: TemplateChild<ProjectHeader>,
        #[template_child]
        pub sidebar: TemplateChild<Sidebar>,
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
                imp.project_header.open_project(&win.project());
                // TODO: project_lists.open_project()
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                settings: gio::Settings::new("ir.imansalmani.iplan.State"),
                project: RefCell::new(Project::default()),
                project_layout_button: TemplateChild::default(),
                project_header: TemplateChild::default(),
                sidebar: TemplateChild::default(),
            }
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
    impl WindowImpl for IPlanWindow {}
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
            let project = create_project("🙂 Personal").expect("Failed to create project");
            create_list("Tasks", project.id()).expect("Failed to create list");
            project
        };
        let window = glib::Object::builder::<IPlanWindow>()
            .property("application", application)
            .property("project", home_project)
            .build();
        let imp = window.imp();

        // Settings
        if imp.settings.int("default-project-layout") == 1 {
            imp.project_layout_button
                .set_icon_name("view-columns-symbolic")
        }
        imp.settings.bind("width", &window, "default-width").build();
        imp.settings
            .bind("width", &window, "default-height")
            .build();
        imp.settings
            .bind("is-maximized", &window, "maximized")
            .build();
        imp.settings
            .bind("is-fullscreen", &window, "fullscreened")
            .build();

        // install Actions
        // let project_actions = gio::SimpleActionGroup::new();
        // window.insert_action_group("project", Some(&project_actions));

        // let action_project_open = gio::SimpleAction::new("open", None);
        // action_project_open.connect_activate(glib::clone!( @weak window as obj => move |_, row| {

        // }));
        // project_actions.add_action(&action_project_open);

        // let action_project_edit = gio::SimpleAction::new("edit", None);
        // action_project_edit.connect_activate(|_, _| {
        //     // TODO: present ProjectEditWindow
        // });
        // project_actions.add_action(&action_project_edit);

        // let action_project_update = gio::SimpleAction::new("update", None);
        // action_project_update.connect_activate(|_, _| {
        //     // TODO: project_header.open_project()
        //     // TODO: sidebar.projects_section.update_project()
        // });
        // project_actions.add_action(&action_project_update);

        // let action_project_delete = gio::SimpleAction::new("delete", Some(glib::VariantTy::INT64));
        // action_project_delete.connect_activate(|_, _| {
        //     // TODO: sidebar.projects_section.handle_project_delete()
        // });
        // project_actions.add_action(&action_project_delete);

        // let list_actions = gio::SimpleActionGroup::new();
        // window.insert_action_group("list", Some(&list_actions));

        // let action_list_new = gio::SimpleAction::new("new", None);
        // action_list_new.connect_activate(|_, _| {
        //     // TODO: project_lists.handle_list_new()
        // });
        // list_actions.add_action(&action_list_new);

        // let search_actions = gio::SimpleActionGroup::new();
        // window.insert_action_group("search", Some(&search_actions));

        // let action_search_window = gio::SimpleAction::new("window", None);
        // action_search_window.connect_activate(glib::clone!( @weak window => move |_, _| {
        //     let active_window = window.application().unwrap().active_window().unwrap();
        //     match active_window.widget_name().as_str() {
        //         "IPlanWindow" => {
        //             // TODO: Present SearchWindow
        //         },
        //         "SearchWindow" => {
        //             active_window.close();
        //         },
        //         _ => {}
        //     }
        // }));
        // search_actions.add_action(&action_search_window);
        // window
        //     .application()
        //     .unwrap()
        //     .set_accels_for_action("search.window", &["<Ctrl>F"]);

        // let action_search_task = gio::SimpleAction::new("task", Some(glib::VariantTy::INT64));
        // action_search_task.connect_activate(|_, _property| {
        //     // TODO: project_header.open_project()
        //     // TODO: project_lists.open_project(task_id)
        //     // TODO: sidebar.projects_section.select_active_project()
        // });
        // search_actions.add_action(&action_search_task);

        // TODO: open active project
        // WidgetExt::activate_action(&window, "project.open", None)
        window
            .activate_action("project.open", None)
            .expect("Failed to open project");
        imp.sidebar.imp().projects_section.select_active_project();

        window
    }

    pub fn project(&self) -> Project {
        self.property("project")
    }

    #[template_callback]
    fn handle_project_layout_button_clicked(&self, button: gtk::Button) {
        let imp = self.imp();
        match button.icon_name() {
            Some(icon_name) => {
                if icon_name == "list-symbolic" {
                    button.set_icon_name("view-columns-symbolic");
                    imp.settings
                        .set_int("default-project-layout", 1)
                        .expect("Could not set setting.");
                } else {
                    button.set_icon_name("list-symbolic");
                    imp.settings
                        .set_int("default-project-layout", 0)
                        .expect("Could not set setting.");
                }
            }
            None => button.set_icon_name("list-symbolic"),
        }
    }
}
