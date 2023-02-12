use gtk::{gdk, gio, glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{
    create_list, create_project, read_project, read_projects, update_project,
};
use crate::views::sidebar::SidebarProject;

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
        imp.projects_box.add_controller(&project_drop_target)
    }

    #[template_callback]
    fn handle_projects_box_row_activated(_projects_box: gtk::ListBox, _row: gtk::ListBoxRow) {
        println!("Row activated");
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        let project = create_project("New Project").expect("Failed to create project");
        create_list("Tasks", project.id()).expect("Failed to create list");
        // TODO: set active project
        let row = SidebarProject::new(project);
        let imp = self.imp();
        imp.projects_box.append(&row);
        imp.projects_box.select_row(Some(&row));
        // TODO: activate project.open action
    }

    // TODO: update_project - used by project.update action in window

    // TODO: project_delete_row - used by project.delete action in window

    // TODO: select_active_project

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
            update_project(project.as_ref()).expect("Failed to update project");
        }
        // TODO: select active project
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
}
