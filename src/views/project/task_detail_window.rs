use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
use crate::db::operations::update_task;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_detail_window.ui")]
    pub struct TaskDetailWindow {
        pub task: RefCell<Task>,
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskDetailWindow {
        const NAME: &'static str = "TaskDetailWindow";
        type Type = super::TaskDetailWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskDetailWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecObject::builder::<Task>("task").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "task" => {
                    let value = value.get::<Task>().expect("value must be a Task");
                    self.task.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "task" => self.task.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for TaskDetailWindow {}
    impl WindowImpl for TaskDetailWindow {}
}

glib::wrapper! {
    pub struct TaskDetailWindow(ObjectSubclass<imp::TaskDetailWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl TaskDetailWindow {
    pub fn new(application: &gtk::Application, app_window: &gtk::Window, task: Task) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.name_entry_row.set_text(&task.name());
        imp.task.replace(task);
        win
    }

    pub fn task(&self) -> Task {
        self.property("task")
    }

    #[template_callback]
    fn handle_name_entry_row_apply(&self, entry_row: adw::EntryRow) {
        let task = self.task();
        task.set_property("name", entry_row.text());
        update_task(task).expect("Failed to update task");
    }
}
