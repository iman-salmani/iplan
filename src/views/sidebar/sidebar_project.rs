use gtk::{gdk, glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{new_position, read_lists, update_task};
use crate::views::project::ProjectListTask;
use crate::views::sidebar::SidebarProjects;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/sidebar/sidebar_project.ui")]
    pub struct SidebarProject {
        pub project: RefCell<Project>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SidebarProject {
        const NAME: &'static str = "SidebarProject";
        type Type = super::SidebarProject;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SidebarProject {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().init_widgets();
        }

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
    impl WidgetImpl for SidebarProject {}
    impl ListBoxRowImpl for SidebarProject {}
}

glib::wrapper! {
    pub struct SidebarProject(ObjectSubclass<imp::SidebarProject>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl SidebarProject {
    pub fn new(project: Project) -> Self {
        let obj = glib::Object::new::<SidebarProject>(&[("project", &project)]);

        let imp = obj.imp();

        imp.name_label.set_text(&obj.project().name());

        if obj.project().archive() {
            imp.name_label.add_css_class("dim-label")
        };

        obj
    }

    pub fn project(&self) -> Project {
        self.property("project")
    }

    fn init_widgets(&self) {
        let task_drop_target =
            gtk::DropTarget::new(ProjectListTask::static_type(), gdk::DragAction::MOVE);
        task_drop_target.set_preload(true);
        task_drop_target.connect_drop(glib::clone!(
            @weak self as obj => @default-return false,
            move |target, value, x, y| obj.task_drop_target_drop(target, value, x, y)));
        task_drop_target.connect_motion(glib::clone!(
            @weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_motion(target, x, y)));
        self.add_controller(&task_drop_target);
    }

    #[template_callback]
    fn handle_drag_prepare(&self, _x: f64, _y: f64) -> Option<gdk::ContentProvider> {
        Some(gdk::ContentProvider::for_value(&self.to_value()))
    }

    #[template_callback]
    fn handle_drag_begin(&self, drag: gdk::Drag) {
        self.parent()
            .unwrap()
            .downcast::<gtk::ListBox>()
            .unwrap()
            .select_row(Some(self));
        let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(&drag).downcast().unwrap();
        let label = gtk::Label::builder().label("label").build();
        drag_icon.set_child(Some(&label));
        drag.set_hotspot(0, 0);
    }

    #[template_callback]
    fn handle_drag_cancel(&self, _drag: gdk::Drag) -> bool {
        // TODO: select active project
        false
    }

    fn task_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        let row: ProjectListTask = value.get().unwrap();
        let task = row.task();
        let project_id = self.project().id();
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
        self.parent()
            .unwrap()
            .parent()
            .and_downcast::<SidebarProjects>()
            .unwrap()
            .select_active_project();
        true
    }

    fn task_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        _y: f64,
    ) -> gdk::DragAction {
        let task_row: ProjectListTask = target.value_as().unwrap();
        if task_row.task().project() != self.project().id() {
            self.parent()
                .and_downcast::<gtk::ListBox>()
                .unwrap()
                .select_row(Some(self));
            gdk::DragAction::MOVE
        } else {
            gdk::DragAction::empty()
        }
    }
}

