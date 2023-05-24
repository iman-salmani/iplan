use gettextrs::gettext;
use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{
    create_list, create_project, new_position, read_lists, read_project, read_projects,
    update_project, update_task,
};
use crate::views::{project::TaskRow, sidebar::SidebarProject, IPlanWindow};
mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/sidebar/sidebar_projects.ui")]
    pub struct SidebarProjects {
        #[template_child]
        pub archive_toggle_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub projects_box: TemplateChild<gtk::ListBox>,
        pub projects: RefCell<Option<gio::ListStore>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarProjects {
        const NAME: &'static str = "SidebarProjects";
        type Type = super::SidebarProjects;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SidebarProjects {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().init_widgets();
        }
    }
    impl WidgetImpl for SidebarProjects {}
    impl BoxImpl for SidebarProjects {}
}

glib::wrapper! {
    pub struct SidebarProjects(ObjectSubclass<imp::SidebarProjects>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl SidebarProjects {
    pub fn new() -> Self {
        glib::Object::new::<SidebarProjects>(&[])
    }

    pub fn select_active_project(&self) {
        // Finding with id because index realtime changes when dragging a project
        let project_id = self
            .root()
            .unwrap()
            .downcast::<IPlanWindow>()
            .unwrap()
            .project()
            .id();
        let projects_box = &self.imp().projects_box;
        for project_row in projects_box.observe_children().into_iter() {
            let project_row: SidebarProject = project_row.unwrap().downcast().unwrap();
            if project_id == project_row.project().id() {
                projects_box.select_row(Some(&project_row));
                break;
            }
        }
    }

    pub fn update_project(&self, project: &Project) {
        let row = self
            .imp()
            .projects_box
            .row_at_index(project.index())
            .and_downcast::<SidebarProject>()
            .unwrap();
        let row_imp = row.imp();
        row_imp.icon_label.set_label(&project.icon());
        row_imp.name_label.set_label(&project.name());
        if project.archive() {
            row_imp.name_label.add_css_class("dim-label");
        } else {
            row_imp.name_label.remove_css_class("dim-label");
        }
        row.set_property("project", project);
        row.changed();
    }

    pub fn delete_project(&self, index: i32) {
        let imp = self.imp();
        let target_row = imp.projects_box.row_at_index(index).unwrap();
        let last_index = imp
            .projects_box
            .last_child()
            .and_downcast::<gtk::ListBoxRow>()
            .unwrap()
            .index();

        for i in index + 1..last_index + 1 {
            let row = imp
                .projects_box
                .row_at_index(i)
                .and_downcast::<SidebarProject>()
                .unwrap();
            let project = row.project();
            project.set_property("index", project.index() - 1);
        }
        imp.projects_box.remove(&target_row);
    }

    pub fn check_archive_hidden(&self) {
        // Filter again maybe previous choice is archived project
        let imp = self.imp();
        if !imp.archive_toggle_button.is_active() {
            imp.projects_box.invalidate_filter();
        }
    }

    fn init_widgets(&self) {
        let imp = self.imp();

        // Fetch
        let projects = read_projects(true).expect("Failed to read projects");
        for project in projects {
            imp.projects_box.append(&SidebarProject::new(project));
        }

        // Projcets box filter
        imp.projects_box.set_filter_func(glib::clone!(
            @weak self as obj => @default-return false,
            move |row| obj.projects_box_filter(row)));

        // Projcets box sort
        imp.projects_box.set_sort_func(|row1, row2| {
            let row1_i = row1.property::<Project>("project").index();
            let row2_i = row2.property::<Project>("project").index();

            if row1_i > row2_i {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });

        // Project drop target
        let project_drop_target =
            gtk::DropTarget::new(SidebarProject::static_type(), gdk::DragAction::MOVE);
        project_drop_target.set_preload(true);
        project_drop_target.connect_drop(glib::clone!(
            @weak self as obj => @default-return false,
            move |target, value, x, y| obj.project_drop_target_drop(target, value, x, y)));
        project_drop_target.connect_motion(glib::clone!(
            @weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.project_drop_target_motion(target, x, y)));
        project_drop_target.connect_enter(glib::clone!(
        @weak self as obj => @default-return gdk::DragAction::empty(),
        move |target, _x, _y| {
            let source_row: Option<SidebarProject> = target.value_as();
            obj.imp().projects_box.select_row(source_row.as_ref());
            gdk::DragAction::MOVE
        }));
        imp.projects_box.add_controller(&project_drop_target);

        // Task drop target
        let task_drop_target = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        task_drop_target.set_preload(true);
        task_drop_target.connect_drop(glib::clone!(
            @weak self as obj => @default-return false,
            move |target, value, x, y| obj.task_drop_target_drop(target, value, x, y)));
        task_drop_target.connect_motion(glib::clone!(
            @weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_motion(target, x, y)));
        task_drop_target.connect_leave(glib::clone!(
        @weak self as obj => move |target| {
            if target.value_as::<TaskRow>().is_some() {
                obj.select_active_project();
            }}));
        imp.projects_box.add_controller(&task_drop_target);
    }

    #[template_callback]
    fn handle_projects_box_row_activated(&self, row: gtk::ListBoxRow) {
        let window = self.root().unwrap().downcast::<IPlanWindow>().unwrap();
        let row = row.downcast::<SidebarProject>().unwrap();
        if window.project().id() != row.project().id() {
            window.set_property("project", row.project().to_value());
            self.activate_action("project.open", None)
                .expect("Failed to open project");
        }
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        let project = create_project("").expect("Failed to create project");
        create_list(&gettext("Tasks"), project.id()).expect("Failed to create list");
        let row = SidebarProject::new(project.clone());
        let imp = self.imp();
        imp.projects_box.append(&row);
        imp.projects_box.select_row(Some(&row));
        self.root()
            .and_downcast::<IPlanWindow>()
            .unwrap()
            .set_property("project", project);
        self.activate_action("project.new", None)
            .expect("Failed to start project.new action");
    }

    #[template_callback]
    fn handle_archive_toggle_button_toggled(&self, _toggle_button: gtk::ToggleButton) {
        self.imp().projects_box.invalidate_filter();
    }

    fn projects_box_filter(&self, row: &gtk::ListBoxRow) -> bool {
        let imp = self.imp();
        if imp.archive_toggle_button.is_active() {
            return true;
        }
        let project = row.property::<Project>("project");
        if !project.archive() {
            return true;
        }
        if let Some(selected_row) = imp.projects_box.selected_row() {
            if selected_row.property::<Project>("project").id() == project.id() {
                return true;
            }
        }
        false
    }

    fn project_drop_target_drop(
        &self,
        target: &gtk::DropTarget,
        _value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        // Source_row moved by motion signal so it should drop on itself
        let row: SidebarProject = target.value_as().unwrap();
        let project = row.project();
        let project_db = read_project(project.id()).expect("Failed to read project");
        if project_db.index() != project.index() {
            update_project(&project).expect("Failed to update project");
        }
        self.select_active_project();
        true
    }

    fn project_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        y: f64,
    ) -> gdk::DragAction {
        let imp = self.imp();
        let source_row: Option<SidebarProject> = target.value_as();
        let target_row = imp.projects_box.row_at_y(y as i32);

        if source_row.is_none() || target_row.is_none() {
            return gdk::DragAction::empty();
        }

        let source_row = source_row.unwrap();
        let target_row: SidebarProject = target_row.unwrap().downcast().unwrap();

        if source_row != target_row {
            let source_i = source_row.index();
            let source_project = source_row.project();
            let target_i: i32 = target_row.index();
            let target_project = target_row.project();

            if source_i - target_i == 1 {
                source_project.set_property("index", source_i - 1);
                target_project.set_property("index", target_i + 1);
                // source_row.set_property(proj, value)
            } else if source_i > target_i {
                for i in target_i..source_i {
                    let row: SidebarProject = imp
                        .projects_box
                        .row_at_index(i)
                        .unwrap()
                        .downcast()
                        .unwrap();
                    row.project().set_property("index", row.index() + 1);
                }
                source_project.set_property("index", target_i)
            } else if source_i - target_i == -1 {
                source_project.set_property("index", source_i + 1);
                target_project.set_property("index", target_i - 1);
            } else if source_i < target_i {
                for i in target_i + 1..source_i + 1 {
                    let row: SidebarProject = imp
                        .projects_box
                        .row_at_index(i)
                        .unwrap()
                        .downcast()
                        .unwrap();
                    row.project().set_property("index", row.index() - 1);
                }
                source_project.set_property("index", target_i)
            }

            imp.projects_box.invalidate_sort();
        }

        gdk::DragAction::MOVE
    }

    fn task_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        value: &glib::Value,
        _x: f64,
        y: f64,
    ) -> bool {
        let row: TaskRow = value.get().unwrap();
        let task = row.task();
        let project_row = self.imp().projects_box.row_at_y(y as i32).unwrap();
        let project_id = project_row.property::<Project>("project").id();
        task.set_property("project", project_id);
        let list_id = read_lists(project_id)
            .expect("Failed to read lists")
            .first()
            .expect("Project should have list")
            .id();
        task.set_property("list", list_id);
        task.set_property("position", new_position(list_id));
        row.parent()
            .and_downcast::<gtk::ListBox>()
            .unwrap()
            .remove(&row);
        update_task(task).expect("Failed to update task");
        self.select_active_project();
        true
    }

    fn task_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        y: f64,
    ) -> gdk::DragAction {
        let task_row: TaskRow = target.value_as().unwrap();
        let projects_box = &self.imp().projects_box;
        if let Some(project_row) = projects_box.row_at_y(y as i32) {
            if task_row.task().project() != project_row.property::<Project>("project").id() {
                projects_box.select_row(Some(&project_row));
                return gdk::DragAction::MOVE;
            }
        }
        gdk::DragAction::empty()
    }
}
