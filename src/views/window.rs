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
use gtk::glib::FromVariant;
use gtk::{gdk, gio, glib, glib::Properties, prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::models::{Project, Task};
use crate::db::operations::{create_project, create_section, read_projects};
use crate::views::project::{ProjectEditWindow, ProjectLayout, ProjectPage};
use crate::views::snippets::MenuItem;
use crate::views::{calendar::Calendar, sidebar::SidebarProjects};

#[derive(PartialEq)]
pub enum ActionScope {
    DeleteToast,
    Project(i64),
    Calendar,
    None,
}

impl StaticVariantType for ActionScope {
    fn static_variant_type() -> std::borrow::Cow<'static, glib::VariantTy> {
        std::borrow::Cow::from(glib::VariantTy::new("ax").unwrap())
    }
}

impl ToVariant for ActionScope {
    fn to_variant(&self) -> glib::Variant {
        let data: [i64; 2] = match self {
            ActionScope::None => [0, 0],
            ActionScope::Calendar => [1, 0],
            ActionScope::Project(id) => [2, id.clone()],
            ActionScope::DeleteToast => [3, 0],
        };
        glib::Variant::array_from_fixed_array(&data)
    }
}

impl FromVariant for ActionScope {
    fn from_variant(variant: &glib::Variant) -> Option<Self> {
        let data = variant.fixed_array::<i64>().ok()?;
        let id = data.get(0)?;
        match id {
            0 => Some(ActionScope::None),
            1 => Some(ActionScope::Calendar),
            2 => Some(ActionScope::Project(data.get(1)?.clone())),
            3 => Some(ActionScope::DeleteToast),
            _ => None,
        }
    }
}

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
        pub stack_pages: TemplateChild<gtk::Stack>,
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
                let modal = ProjectEditWindow::new(obj.application().unwrap(), obj, obj.project());
                modal.connect_closure(
                    "changed",
                    true,
                    glib::closure_local!(@watch obj => move |_: ProjectEditWindow, project: Project| {
                        if let Some(page) = obj.visible_project_page() {
                            page.imp().project_header.open_project(&project);
                        }
                        obj.imp().sidebar_projects.update_project(&project);
                        obj.set_project(project);
                    })
                );
                modal.present();
            });
            klass.install_action("project.delete", None, move |obj, _, _| {
                let projects_section = &obj.imp().sidebar_projects;
                projects_section.delete_project(obj.project().index());
                let home_project = obj.home_project();
                obj.imp()
                    .stack_pages
                    .remove(&obj.visible_project_page().unwrap());
                obj.change_project(home_project);
            });
            klass.install_action("section.new", None, move |obj, _, _| {
                obj.visible_project_page()
                    .unwrap()
                    .new_section(obj.project().id());
            });
            klass.install_action(
                "task.changed",
                Some(&format!(
                    "({}{})",
                    Task::static_variant_type().as_str(),
                    ActionScope::static_variant_type().as_str()
                )),
                |obj, _, value| {
                    let (task, scope): (Task, ActionScope) = value.unwrap().get().unwrap();
                    let imp = obj.imp();

                    let update_project_page = || {
                        let page_name = task.project().to_string();
                        if let Some(page) = imp.stack_pages.child_by_name(&page_name) {
                            let page = page.downcast::<ProjectPage>().unwrap();
                            page.reset_task(task.clone());
                        }
                    };

                    match scope {
                        ActionScope::DeleteToast => {
                            task.set_suspended(false);
                            update_project_page();
                            imp.calendar.set_subtasks_suspended(task.id(), false);
                            imp.calendar.reset_task(task);
                        }
                        ActionScope::Project(_) => imp.calendar.reset_task(task),
                        ActionScope::Calendar => update_project_page(),
                        ActionScope::None => {
                            update_project_page();
                            imp.calendar.reset_task(task);
                        }
                    }
                },
            );
            klass.install_action(
                "task.duration-changed",
                Some(&format!(
                    "({}{})",
                    Task::static_variant_type().as_str(),
                    ActionScope::static_variant_type().as_str()
                )),
                move |obj: &super::IPlanWindow, _, value| {
                    let (task, scope): (Task, ActionScope) = value.unwrap().get().unwrap();
                    let imp = obj.imp();

                    let update_project_page = || {
                        let page_name = task.project().to_string();
                        if let Some(page) = imp.stack_pages.child_by_name(&page_name) {
                            let page = page.downcast::<ProjectPage>().unwrap();
                            page.refresh_task_timer(task.clone());
                            page.imp().project_header.set_stat_updated(false);
                        }
                    };

                    match scope {
                        ActionScope::Project(_) => {
                            let task_id = task.id();
                            imp.calendar.refresh_task_timer(task_id);
                            imp.calendar.refresh_days_views_duration(task_id);
                            imp.calendar.refresh_parents_timers(task.parent());
                        }
                        ActionScope::Calendar => update_project_page(),
                        ActionScope::None => {
                            update_project_page();
                            let task_id = task.id();
                            imp.calendar.refresh_task_timer(task_id);
                            imp.calendar.refresh_days_views_duration(task_id);
                            imp.calendar.refresh_parents_timers(task.parent());
                        }
                        _ => unimplemented!(),
                    }
                },
            );
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
        let project_page = self
            .project_by_id(project_id)
            .unwrap_or_else(|| self.new_project_page(&project));
        self.set_visible_project(project_id);
        self.apply_project_layout();
        project_page.select_task(None);
        imp.sidebar_projects.check_archive_hidden();
        imp.sidebar_projects.select_active_project();
        imp.calendar_button.add_css_class("flat");
    }

    pub fn visible_project_page(&self) -> Option<ProjectPage> {
        self.imp()
            .stack_pages
            .visible_child()
            .and_downcast::<ProjectPage>()
    }

    pub fn close_sidebar(&self) {
        let imp = self.imp();
        if imp.flap.is_folded() {
            imp.flap.set_reveal_flap(false);
        }
    }

    pub fn reset(&self) {
        let imp = self.imp();

        let pages = imp.stack_pages.observe_children();
        for _ in 0..pages.n_items() {
            let page = &pages.item(0).and_downcast::<gtk::Widget>().unwrap();
            imp.stack_pages.remove(page);
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
        if let Some(child) = self.imp().stack_pages.child_by_name(&id.to_string()) {
            child.downcast::<ProjectPage>().ok()
        } else {
            None
        }
    }

    fn new_project_page(&self, project: &Project) -> ProjectPage {
        let imp = self.imp();
        let project_page = ProjectPage::new();
        let project_page_imp = project_page.imp();

        project_page_imp.layout_button.connect_clicked(
            glib::clone!(@weak self as obj => move |button| {
                let layout = if button.icon_name().unwrap() == "list-symbolic" { 1 } else { 0 };
                obj.set_project_layout(layout);
                obj.apply_project_layout();
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

        imp.stack_pages
            .add_named(&project_page, Some(&project.id().to_string()));

        project_page.open_project(project);
        project_page
    }

    fn set_visible_project(&self, id: i64) {
        self.imp()
            .stack_pages
            .set_visible_child_name(&id.to_string());
    }

    fn apply_project_layout(&self) {
        if self.project_layout() == 1 {
            self.visible_project_page()
                .unwrap()
                .set_layout(ProjectLayout::Horizontal);
        } else {
            self.visible_project_page()
                .unwrap()
                .set_layout(ProjectLayout::Vertical);
        }
    }

    #[template_callback]
    fn handle_calendar_button_clicked(&self, button: MenuItem) {
        let imp = self.imp();
        self.close_sidebar();

        if self.visible_project_page().is_none() {
            return;
        }

        button.remove_css_class("flat");
        imp.project.take();
        self.imp().stack_pages.set_visible_child_name("calendar");
        let projects_box: &gtk::ListBox = imp.sidebar_projects.imp().projects_box.as_ref();
        if let Some(row) = projects_box.selected_row() {
            projects_box.unselect_row(&row);
        }
    }
}
