use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::unimplemented;

use crate::db::models::Task;
use crate::db::operations::read_task;
use crate::views::project::TaskPage;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_window.ui")]
    pub struct TaskWindow {
        pub prev_task: Cell<i64>,
        #[template_child]
        pub back_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub back_button_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub task_pages_stack: TemplateChild<gtk::Stack>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskWindow {
        const NAME: &'static str = "TaskWindow";
        type Type = super::TaskWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("project.update", None, move |obj, _, _value| {
                obj.transient_for()
                    .unwrap()
                    .activate_action("project.update", None)
                    .expect("Failed to send project.update action");
            });
            klass.install_action("subtask.open", Some("i"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get().unwrap();
                let task = read_task(value).expect("Failed to read task");
                let task_id = task.id().to_string();
                let visible_task_page = imp
                    .task_pages_stack
                    .visible_child()
                    .and_downcast::<TaskPage>()
                    .unwrap();
                let parent_task = visible_task_page.task();
                obj.set_property("prev-task", parent_task.id());
                imp.task_pages_stack
                    .add_named(&TaskPage::new(task), Some(&task_id));
                imp.task_pages_stack
                    .set_visible_child_full(&task_id, gtk::StackTransitionType::SlideLeft);
                imp.back_button_content.set_label(&parent_task.name());
                imp.back_button.set_visible(true);
            });
            klass.install_action("record.created", Some("x"), move |obj, _, value| {
                let record_id = value.unwrap().get::<i64>().unwrap();
                let imp = obj.imp();
                imp.task_pages_stack
                    .visible_child()
                    .and_downcast::<TaskPage>()
                    .unwrap()
                    .add_record(record_id);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecInt64::builder("prev-task").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "prev-task" => {
                    let value = value.get::<i64>().expect("value must be a Task");
                    self.prev_task.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "prev-task" => self.prev_task.get().to_value(),
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
        let task_id = task.id().to_string();
        imp.task_pages_stack
            .add_named(&TaskPage::new(task), Some(&task_id));
        win
    }

    pub fn prev_task(&self) -> i64 {
        self.property("prev-task")
    }

    #[template_callback]
    fn handle_back_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let suspended_page = imp.task_pages_stack.visible_child().unwrap();
        imp.task_pages_stack.set_visible_child_full(
            &self.prev_task().to_string(),
            gtk::StackTransitionType::SlideRight,
        );
        imp.task_pages_stack.remove(&suspended_page);
        let visible_task_page = imp
            .task_pages_stack
            .visible_child()
            .and_downcast::<TaskPage>()
            .unwrap();
        let task = visible_task_page.task();
        let parent_id = task.parent();
        self.set_property("prev-task", parent_id);
        if parent_id == 0 {
            imp.back_button.set_visible(false);
        } else {
            let parent_task = read_task(parent_id).expect("Failed to read task");
            imp.back_button_content.set_label(&parent_task.name());
        }
    }
}
