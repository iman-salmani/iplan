use gtk::{gdk, glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::{Ref, RefCell};

use crate::db::models::Project;

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

    pub fn project(&self) -> Ref<Project> {
        self.imp().project.borrow()
    }

    fn init_widgets(&self) {}

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

    // TODO: handle_drop_task_target_drop

    // TODO: handle_drop_task_target_motion
}
