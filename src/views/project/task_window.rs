use adw::traits::ExpanderRowExt;
use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
use crate::db::operations::{create_task, read_tasks, update_task};
use crate::views::project::TaskRow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_window.ui")]
    pub struct TaskWindow {
        pub task: RefCell<Task>,
        #[template_child]
        pub task_row: TemplateChild<TaskRow>,
        #[template_child]
        pub description_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub description_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub subtasks_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskWindow {
        const NAME: &'static str = "TaskWindow";
        type Type = super::TaskWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, _value| {
                let imp = obj.imp();
                imp.subtasks_box.invalidate_sort();
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

    impl ObjectImpl for TaskWindow {
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
    impl WidgetImpl for TaskWindow {}
    impl WindowImpl for TaskWindow {}
}

glib::wrapper! {
    pub struct TaskWindow(ObjectSubclass<imp::TaskWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl TaskWindow {
    pub fn new(application: &gtk::Application, app_window: &gtk::Window, task: Task) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.task_row.set_property("task", task.clone());
        imp.task_row.init_widgets();
        let task_description = task.description();
        imp.description_expander_row.set_subtitle(&task_description);
        imp.description_buffer.set_text(&task_description);
        imp.subtasks_box.set_sort_func(|row1, _row2| {
            if row1.property::<Task>("task").done() {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });
        let tasks = read_tasks(task.project(), None, None, Some(task.id()))
            .expect("Failed to read subtasks");
        for task in tasks {
            let row = TaskRow::new(task);
            row.init_widgets();
            imp.subtasks_box.append(&row);
        }
        imp.task.replace(task);
        win
    }

    pub fn task(&self) -> Task {
        self.property("task")
    }

    #[template_callback]
    fn handle_description_buffer_changed(&self, buffer: gtk::TextBuffer) {
        let task = self.task();
        task.set_property(
            "description",
            buffer.text(&buffer.start_iter(), &buffer.end_iter(), true),
        );
        update_task(task).expect("Failed to update task");
    }

    #[template_callback]
    fn handle_new_subtask_button_clicked(&self, _button: gtk::Button) {
        let task = self.task();
        let task = create_task("", task.project(), 0, task.id()).expect("Failed to create subtask");
        let task_ui = TaskRow::new(task);
        let imp = self.imp();
        imp.subtasks_box.prepend(&task_ui);
        task_ui.init_widgets();
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }
}
