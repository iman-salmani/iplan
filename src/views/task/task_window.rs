use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::unimplemented;

use crate::application::IPlanApplication;
use crate::db::models::Task;
use crate::db::operations::read_task;
use crate::views::task::{TaskPage, TasksDoneWindow};
use crate::views::{ActionScope, IPlanWindow};
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
                let visible_task_page = obj.visible_page();
                let parent_task = visible_task_page.task();
                obj.set_property("parent-task", parent_task.id());
                imp.task_pages_stack
                    .add_named(&TaskPage::new(task), Some(&task_id));
                imp.task_pages_stack
                    .set_visible_child_full(&task_id, gtk::StackTransitionType::SlideLeft);
                imp.back_button_content.set_label(&parent_task.name());
                imp.back_button.set_visible(true);
            });
            klass.install_action("reminder.created", Some("x"), move |obj, _, value| {
                let reminder_id = value.unwrap().get::<i64>().unwrap();
                obj.visible_page().add_reminder(reminder_id);
            });
            klass.install_action(
                "task.changed",
                Some(&Task::static_variant_type().as_str()),
                move |obj, _, value| {
                    let task: Task = value.unwrap().get().unwrap();
                    obj.emit_by_name::<()>("task-changed", &[&task]);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("window-closed")
                        .param_types([Task::static_type()])
                        .build(),
                    Signal::builder("page-closed")
                        .param_types([Task::static_type()])
                        .build(),
                    Signal::builder("task-changed")
                        .param_types([Task::static_type()])
                        .build(),
                    Signal::builder("task-duration-changed")
                        .param_types([Task::static_type()])
                        .build(),
                ]
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
            let mut root_task = None;
            let pages = self.task_pages_stack.pages();
            for i in 0..pages.n_items() {
                let stack_page = pages.item(i).and_downcast::<gtk::StackPage>().unwrap();
                let page = stack_page.child().downcast::<TaskPage>().unwrap();
                let task = page.task();
                if task.parent() == 0 {
                    root_task = Some(task);
                }
                obj.emit_by_name::<()>("page-closed", &[&page.task()]);
            }

            if root_task.is_none() {
                let page = self.obj().visible_page();
                let mut task = page.task();
                let mut parent_id = task.parent();
                while parent_id != 0 {
                    let parent_name = parent_id.to_string();
                    task = if let Some(page) = self.task_pages_stack.child_by_name(&parent_name) {
                        page.downcast::<TaskPage>().unwrap().task()
                    } else {
                        read_task(parent_id).unwrap()
                    };
                    parent_id = task.parent();
                }
                root_task = Some(task);
            }

            obj.emit_by_name::<()>("window-closed", &[&root_task.unwrap()]);

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
            let parent_task = read_task(parent_id).unwrap();
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

    pub fn add_delete_toast(&self, task: &Task, toast: adw::Toast) {
        let imp = self.imp();
        let task_parent = task.parent();
        let page_task_parent = imp.parent_task.get();
        if page_task_parent != task_parent {
            if page_task_parent == 0 {
                toast.connect_button_clicked(
                    glib::clone!(@weak self as obj, @strong task => move |_toast| {
                        obj.direct_subtask_undo_delete(&task);
                    }),
                );
            }
            imp.toast_overlay.add_toast(toast);
            return;
        }

        if task_parent == 0 {
            let transient_for = self.transient_for().unwrap();
            let transient_for_name = transient_for.widget_name();
            if transient_for_name == "IPlanWindow" {
                let transient_for = transient_for.downcast::<IPlanWindow>().unwrap();
                transient_for.imp().toast_overlay.add_toast(toast);
            } else if transient_for_name == "TasksDoneWindow" {
                let transient_for = transient_for.downcast::<TasksDoneWindow>().unwrap();
                transient_for.add_delete_toast(&task, toast);
            }
            self.close();
        } else {
            imp.back_button.emit_clicked();
            toast.connect_button_clicked(
                glib::clone!(@weak self as obj, @weak task => move |_toast| {
                    obj.direct_subtask_undo_delete(&task);
                }),
            );
            self.imp().toast_overlay.add_toast(toast);
        }
    }

    fn visible_page(&self) -> TaskPage {
        self.imp()
            .task_pages_stack
            .visible_child()
            .and_downcast::<TaskPage>()
            .unwrap()
    }

    fn direct_subtask_undo_delete(&self, task: &Task) {
        let task_page: TaskPage = self.visible_page();
        if let Some(row) = task_page.imp().subtasks_box.item_by_id(task.id()) {
            row.task().set_suspended(false);
            row.changed();
        }
        let app = self
            .application()
            .and_downcast::<IPlanApplication>()
            .unwrap();
        let main_win: gtk::Window = app.window_by_name("IPlanWindow").unwrap();
        main_win
            .activate_action(
                "task.changed",
                Some(&glib::Variant::from((
                    task.to_variant(),
                    ActionScope::DeleteToast.to_variant(),
                ))),
            )
            .unwrap();

        let transient_for = self.transient_for().unwrap();
        if transient_for.widget_name() == "TasksDoneWindow" {
            let transient_for = transient_for.downcast::<TasksDoneWindow>().unwrap();
            if let Some(row) = transient_for.row_by_id(task.parent()) {
                row.reset_subtasks();
            }
        }
    }

    #[template_callback]
    fn handle_back_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let from_page = self.visible_page();
        let from_task = from_page.task();
        let parent_name = self.parent_task().to_string();
        let target_page = if let Some(page) = imp.task_pages_stack.child_by_name(&parent_name) {
            page.downcast().unwrap()
        } else {
            let parent_task = read_task(from_task.parent()).expect("Failed to read task");
            let page = TaskPage::new(parent_task);
            imp.task_pages_stack.add_named(&page, Some(&parent_name));
            page
        };

        imp.task_pages_stack
            .set_visible_child_full(&parent_name, gtk::StackTransitionType::SlideRight);
        imp.task_pages_stack.remove(&from_page);
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
