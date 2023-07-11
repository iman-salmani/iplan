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
use std::cell::{Cell, RefCell};

use crate::db::models::Project;
use crate::db::operations::{create_project, create_section, read_projects};
use crate::views::project::{ProjectEditWindow, ProjectLayout, ProjectPage};
use crate::views::snippets::MenuItem;
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
        #[property(get, set, name = "project-layout")]
        pub project_layout: Cell<i32>,
        #[template_child]
        pub flap: TemplateChild<adw::Flap>,
        #[template_child]
        pub sidebar_projects: TemplateChild<SidebarProjects>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub projects_stack: TemplateChild<gtk::Stack>,
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
            MenuItem::ensure_type();
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action(
                "project.open",
                Some(&Project::static_variant_type_string()),
                move |obj, _, value| {
                    let project = Project::try_from(value.unwrap()).unwrap();
                    let project_id = project.id();

                    obj.close_sidebar();

                    if obj.project().id() == project_id {
                        return;
                    }

                    obj.change_project(project); // FIXME: Inside check_project called select_active_project. this unnecessary if the action is sent via sidebar projects
                },
            );
            klass.install_action("project.edit", None, move |obj, _, _| {
                let window = ProjectEditWindow::new(obj.application().unwrap(), obj, obj.project());
                window.present();
            });
            klass.install_action("project.update", None, move |obj, _, _| {
                let imp = obj.imp();
                if imp.calendar.is_visible() {
                    return;
                }
                let project = obj.project();
                obj.visible_project_page()
                    .imp()
                    .project_header
                    .open_project(&project);
                imp.sidebar_projects.update_project(&project);
            });
            klass.install_action("project.delete", None, move |obj, _, _| {
                let projects_section = &obj.imp().sidebar_projects;
                projects_section.delete_project(obj.project().index());
                let home_project = obj.home_project();
                obj.imp().projects_stack.remove(&obj.visible_project_page());
                obj.change_project(home_project);
            });
            klass.install_action("section.new", None, move |obj, _, _| {
                obj.visible_project_page().new_section(obj.project().id());
            });
            klass.install_action("task.duration-changed", Some("x"), move |obj, _, _| {
                obj.visible_project_page()
                    .imp()
                    .project_header
                    .set_stat_updated(false);
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
        let settings = gio::Settings::new("ir.imansalmani.IPlan.State");
        let obj = glib::Object::builder::<IPlanWindow>()
            .property("application", application)
            .property("default-width", settings.int("width"))
            .property("default-height", settings.int("height"))
            .property("maximized", settings.boolean("is-maximized"))
            .property("fullscreened", settings.boolean("is-fullscreen"))
            .build();
        let home_project = obj.home_project();
        settings
            .bind("default-project-layout", &obj, "project-layout")
            .build();
        obj.set_settings(settings);
        obj.change_project(home_project);

        if let Some(display) = gdk::Display::default() {
            let provider = gtk::CssProvider::new();
            provider.load_from_resource("/ir/imansalmani/iplan/ui/style.css");
            gtk::style_context_add_provider_for_display(&display, &provider, 400);
        }

        let imp = obj.imp();
        let calendar_imp = imp.calendar.imp();
        calendar_imp
            .toggle_sidebar_button
            .bind_property("active", &imp.flap.get(), "reveal-flap")
            .sync_create()
            .bidirectional()
            .build();

        imp.flap
            .bind_property(
                "folded",
                &calendar_imp.page_header.get(),
                "show-start-title-buttons",
            )
            .sync_create()
            .build();

        imp.flap
            .bind_property(
                "folded",
                &calendar_imp.toggle_sidebar_button.get(),
                "visible",
            )
            .sync_create()
            .build();

        obj
    }

    pub fn change_project(&self, project: Project) {
        let imp = self.imp();
        let project_id = project.id();
        self.set_project(&project);
        let project_page = if let Some(project_page) = self.project_by_id(project_id) {
            self.imp().projects_stack.remove(&project_page);
            self.new_project_page(project_id)
        } else {
            self.new_project_page(project_id)
        };
        self.set_visible_project(project_id);
        self.apply_project_layout();
        project_page.open_project(&project);
        project_page.select_task(None);
        imp.sidebar_projects.check_archive_hidden();
        imp.sidebar_projects.select_active_project();
        self.close_calendar();
    }

    pub fn visible_project_page(&self) -> ProjectPage {
        self.imp()
            .projects_stack
            .visible_child()
            .and_downcast::<ProjectPage>()
            .unwrap()
    }

    pub fn close_sidebar(&self) {
        let imp = self.imp();
        if imp.flap.is_folded() {
            imp.flap.set_reveal_flap(false);
        }
    }

    pub fn reset(&self) {
        let imp = self.imp();

        let pages = imp.projects_stack.observe_children();
        for _ in 0..pages.n_items() {
            let page = &pages.item(0).and_downcast::<gtk::Widget>().unwrap();
            imp.projects_stack.remove(page);
        }
        imp.calendar.refresh();
        self.set_project(Project::default());

        let home_project = self.home_project();
        imp.sidebar_projects.reset();
        self.change_project(home_project);
    }

    fn home_project(&self) -> Project {
        let projects = read_projects(true).unwrap();
        if let Some(project) = projects.first() {
            project.clone()
        } else {
            let project: Project = create_project(&gettext("Personal"), "", "").unwrap();
            create_section(&gettext("Tasks"), project.id()).unwrap();
            self.imp().sidebar_projects.add_project(project.to_owned());
            project
        }
    }

    fn project_by_id(&self, id: i64) -> Option<ProjectPage> {
        if let Some(child) = self.imp().projects_stack.child_by_name(&id.to_string()) {
            child.downcast::<ProjectPage>().ok()
        } else {
            None
        }
    }

    fn new_project_page(&self, project_id: i64) -> ProjectPage {
        let imp = self.imp();
        let project_page = ProjectPage::new();
        let project_page_imp = project_page.imp();

        project_page_imp.layout_button.connect_clicked(
            glib::clone!(@weak self as obj => move |button| {
                let layout = if button.icon_name().unwrap() == "list-symbolic" { 1 } else { 0 };
                obj.set_project_layout(layout);
                obj.apply_project_layout();
                obj.visible_project_page().open_project(&obj.project());
            }),
        );

        project_page_imp
            .toggle_sidebar_button
            .bind_property("active", &imp.flap.get(), "reveal-flap")
            .sync_create()
            .bidirectional()
            .build();

        self.bind_property(
            "project-layout",
            &project_page_imp.layout_button.get(),
            "icon-name",
        )
        .transform_to(|_, layout: i32| {
            if layout == 1 {
                Some("view-columns-symbolic")
            } else {
                Some("list-symbolic")
            }
        })
        .sync_create()
        .build();

        imp.flap
            .bind_property(
                "folded",
                &project_page_imp.page_header.get(),
                "show-start-title-buttons",
            )
            .sync_create()
            .build();

        imp.flap
            .bind_property(
                "folded",
                &project_page_imp.toggle_sidebar_button.get(),
                "visible",
            )
            .sync_create()
            .build();

        imp.projects_stack
            .add_named(&project_page, Some(&project_id.to_string()));

        project_page
    }

    fn set_visible_project(&self, id: i64) {
        self.imp()
            .projects_stack
            .set_visible_child_full(&id.to_string(), gtk::StackTransitionType::Crossfade);
    }

    fn apply_project_layout(&self) {
        if self.project_layout() == 1 {
            self.visible_project_page()
                .set_layout(ProjectLayout::Horizontal);
        } else {
            self.visible_project_page()
                .set_layout(ProjectLayout::Vertical);
        }
    }

    fn close_calendar(&self) {
        let imp = self.imp();
        imp.calendar.set_visible(false);
        imp.calendar_button.add_css_class("flat");
    }

    #[template_callback]
    fn handle_calendar_button_clicked(&self, button: MenuItem) {
        let imp = self.imp();
        self.close_sidebar();

        if imp.calendar.is_visible() {
            return;
        }

        button.remove_css_class("flat");
        imp.project.take();
        imp.calendar.set_visible(true);
        imp.calendar.refresh();
        let projects_box: &gtk::ListBox = imp.sidebar_projects.imp().projects_box.as_ref();
        projects_box.unselect_row(&projects_box.selected_row().unwrap());
    }
}
