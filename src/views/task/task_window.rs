use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::unimplemented;

use crate::db::models::Task;
use crate::db::operations::read_task;
use crate::views::project::ProjectDoneTasksWindow;
use crate::views::task::{TaskPage, TaskRow};
use crate::views::IPlanWindow;
mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/task/task_window.ui")]
    pub struct TaskWindow {
        pub parent_task: Cell<i64>,
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
                obj.set_property("parent-task", parent_task.id());
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
            klass.install_action("reminder.created", Some("x"), move |obj, _, value| {
                let reminder_id = value.unwrap().get::<i64>().unwrap();
                let imp = obj.imp();
                imp.task_pages_stack
                    .visible_child()
                    .and_downcast::<TaskPage>()
                    .unwrap()
                    .add_reminder(reminder_id);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("task-window-close")
                    .param_types([Task::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecInt64::builder("parent-task").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "parent-task" => {
                    let value = value.get::<i64>().expect("value must be a Task");
                    self.parent_task.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "parent-task" => self.parent_task.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for TaskWindow {}
    impl WindowImpl for TaskWindow {
        fn close_request(&self) -> glib::signal::Inhibit {
            let obj = self.obj();
            let page = obj
                .imp()
                .task_pages_stack
                .visible_child()
                .and_downcast::<TaskPage>()
                .unwrap();
            obj.emit_by_name::<()>("task-window-close", &[&page.task()]);
            self.parent_close_request()
        }
    }
}

glib::wrapper! {
    pub struct TaskWindow(ObjectSubclass<imp::TaskWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl TaskWindow {
    pub fn new<P: glib::IsA<gtk::Window>>(
        application: &gtk::Application,
        app_window: &P,
        task: Task,
    ) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        let task_id = task.id().to_string();
        let parent_id = task.parent();
        if parent_id != 0 {
            let parent_task = read_task(parent_id).expect("Failed to read task");
            win.set_property("parent-task", parent_id);
            imp.back_button_content.set_label(&parent_task.name());
            imp.back_button.set_visible(true);
        }
        imp.task_pages_stack
            .add_named(&TaskPage::new(task), Some(&task_id));
        win
    }

    pub fn parent_task(&self) -> i64 {
        self.property("parent-task")
    }

    pub fn add_toast(&self, task: Task, toast: adw::Toast) {
        let imp = self.imp();
        let task_parent = task.parent();
        if imp.parent_task.get() != task_parent {
            imp.toast_overlay.add_toast(toast);
            return;
        }
        if task_parent == 0 {
            let transient_for = self.transient_for().unwrap();
            let transient_for_name = transient_for.widget_name();
            if transient_for_name == "IPlanWindow" {
                let transient_for = transient_for.downcast::<IPlanWindow>().unwrap();
                toast.connect_button_clicked(glib::clone!(@weak transient_for =>
                    move |_toast| {
                        let calendar = transient_for.imp().calendar.get();
                        if calendar.is_visible() {
                            calendar.refresh();
                        } else {
                            transient_for.activate_action("project.open", None).expect("Failed to send project.open action");
                        }
                }));
                transient_for.imp().toast_overlay.add_toast(toast);
            } else if transient_for_name == "ProjectDoneTasksWindow" {
                let transient_for = transient_for.downcast::<ProjectDoneTasksWindow>().unwrap();
                toast.connect_button_clicked(glib::clone!(@weak task, @weak transient_for =>
                    move |_toast| {
                        let tasks_rows = transient_for.imp().tasks_box.observe_children();
                        for i in 0..tasks_rows.n_items() {
                            let row = tasks_rows.item(i).and_downcast::<TaskRow>().unwrap();
                            if row.task().id() == task.id() {
                                row.task().set_suspended(false);
                                row.changed();
                                break;
                            }
                        }
                }));
                transient_for.imp().toast_overlay.add_toast(toast);
            }
            self.close();
        } else {
            imp.back_button.emit_clicked();
            toast.connect_button_clicked(glib::clone!(@weak imp, @weak task =>
                move |_toast| {
                    let task_page = imp.task_pages_stack.visible_child().and_downcast::<TaskPage>().unwrap();
                    let subtasks_rows = task_page.imp().subtasks_box.observe_children();
                    for i in 0..subtasks_rows.n_items() {
                        let row = subtasks_rows.item(i).and_downcast::<TaskRow>().unwrap();
                        if row.task().id() == task.id() {
                            row.task().set_suspended(false);
                            row.changed();
                            break;
                        }
                    }
            }));
            self.imp().toast_overlay.add_toast(toast);
        }
    }

    #[template_callback]
    fn handle_back_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let from_page = imp
            .task_pages_stack
            .visible_child()
            .and_downcast::<TaskPage>()
            .unwrap();
        let from_task = from_page.task();
        let parent_name = self.parent_task().to_string();
        if imp.task_pages_stack.child_by_name(&parent_name).is_none() {
            let parent_task = read_task(from_task.parent()).expect("Failed to read task");
            imp.task_pages_stack
                .add_named(&TaskPage::new(parent_task), Some(&parent_name));
        }
        imp.task_pages_stack
            .set_visible_child_full(&parent_name, gtk::StackTransitionType::SlideRight);
        imp.task_pages_stack.remove(&from_page);
        let target_page = imp
            .task_pages_stack
            .visible_child()
            .and_downcast::<TaskPage>()
            .unwrap();
        if let Some(task_row) = target_page.imp().subtasks_box.item_by_id(from_task.id()) {
            task_row.reset(from_task);
            task_row.changed();
        }
        let task = target_page.task();
        let parent_id = task.parent();
        let target_page_imp = target_page.imp();
        target_page_imp.task_row.refresh_timer();
        self.set_property("parent-task", parent_id);
        if parent_id == 0 {
            imp.back_button.set_visible(false);
        } else {
            let parent_task = read_task(parent_id).expect("Failed to read task");
            imp.back_button_content.set_label(&parent_task.name());
        }
    }
}
