use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
use crate::db::operations::{create_task, read_tasks};
use crate::views::project::TaskRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/subtasks_window.ui")]
    pub struct SubTasksWindow {
        pub task: RefCell<Task>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubTasksWindow {
        const NAME: &'static str = "SubTasksWindow";
        type Type = super::SubTasksWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, _value| {
                let imp = obj.imp();
                imp.tasks_box.invalidate_sort();
            });
            klass.install_action("project.update", None, move |obj, _, _value| {
                obj.transient_for()
                    .unwrap()
                    .activate_action("project.update", None)
                    .expect("Failed to send project.update action");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubTasksWindow {
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
    impl WidgetImpl for SubTasksWindow {}
    impl WindowImpl for SubTasksWindow {}
}

glib::wrapper! {
    pub struct SubTasksWindow(ObjectSubclass<imp::SubTasksWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl SubTasksWindow {
    pub fn new(application: &gtk::Application, app_window: &gtk::Window, task: Task) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.tasks_box.set_sort_func(|row1, _row2| {
            if row1.property::<Task>("task").done() {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });
        let tasks = read_tasks(task.project(), None, None, Some(task.id()))
            .expect("Failed to read subtasks");
        imp.task.replace(task);
        for task in tasks {
            let row = TaskRow::new(task);
            row.init_widgets();
            imp.tasks_box.append(&row);
        }
        win
    }

    pub fn task(&self) -> Task {
        self.property("task")
    }

    #[template_callback]
    fn handle_new_task_button_clicked(&self, _button: gtk::Button) {
        let task = self.task();
        let task = create_task("", task.project(), 0, task.id()).expect("Failed to create subtask");
        let task_ui = TaskRow::new(task);
        let imp = self.imp();
        imp.tasks_box.prepend(&task_ui);
        task_ui.init_widgets();
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }
}
