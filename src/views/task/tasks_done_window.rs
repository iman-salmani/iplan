use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{glib, glib::Properties};
use std::cell::RefCell;

use crate::db::models::{Section, Task};
use crate::db::operations::{read_task, read_tasks};
use crate::views::task::{TaskRow, TaskWindow};
use crate::views::{ActionScope, IPlanWindow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/task/tasks_done_window.ui")]
    #[properties(wrapper_type=super::TasksDoneWindow)]
    pub struct TasksDoneWindow {
        #[property(get, set)]
        pub section: RefCell<Section>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TasksDoneWindow {
        const NAME: &'static str = "TasksDoneWindow";
        type Type = super::TasksDoneWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action(
                "task.changed",
                Some(&Task::static_variant_type().as_str()),
                move |obj, _, value| {
                    let imp = obj.imp();
                    let task: Task = value.unwrap().get().unwrap();

                    obj.activate_task_action("task.changed", &task);

                    if task.done() {
                        return;
                    }

                    let row = obj.row_by_id(task.id()).unwrap();
                    let upper_row = imp.tasks_box.row_at_index(row.index() - 1);
                    if let Some(upper_row) = upper_row {
                        upper_row.grab_focus();
                    }
                    imp.tasks_box.remove(&row);
                    obj.emit_by_name::<()>("task-undo", &[&task]);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TasksDoneWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("task-undo")
                    .param_types([Task::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }
    impl WidgetImpl for TasksDoneWindow {}
    impl WindowImpl for TasksDoneWindow {}
    impl AdwWindowImpl for TasksDoneWindow {}
}

glib::wrapper! {
    pub struct TasksDoneWindow(ObjectSubclass<imp::TasksDoneWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl TasksDoneWindow {
    pub fn new(application: gtk::Application, app_window: &IPlanWindow, section: Section) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.name_label.set_label(&gettext("Done Tasks"));
        let tasks = read_tasks(
            Some(section.project()),
            Some(section.id()),
            Some(true),
            Some(0),
            None,
            false,
        )
        .unwrap();
        for task in tasks {
            let task_row = TaskRow::new(task, false, false);
            imp.tasks_box.append(&task_row);
        }
        imp.tasks_box.set_sort_func(|row1, row2| {
            let row1_p = row1.property::<Task>("task").position();
            let row2_p = row2.property::<Task>("task").position();

            if row1_p < row2_p {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });

        imp.tasks_box.set_filter_func(glib::clone!(
        @weak imp => @default-return false,
        move |row| {
            let row = row.downcast_ref::<TaskRow>().unwrap();
            if row.task().suspended() {
                false
            } else {
                !row.imp().moving_out.get()
            }
        }));
        win
    }

    pub fn select_task(&self, task_id: i64) {
        let imp = self.imp();
        let tasks = imp.tasks_box.observe_children();
        let task = read_task(task_id).expect("Failed to read task");
        for i in 0..tasks.n_items() - 1 {
            if let Some(task_row) = tasks.item(i).and_downcast::<TaskRow>() {
                if task_row.task().position() == task.position() {
                    task_row.grab_focus();
                    break;
                }
            }
        }
    }

    pub fn row_by_id(&self, id: i64) -> Option<TaskRow> {
        let imp = self.imp();
        let rows = imp.tasks_box.observe_children();
        for i in 0..rows.n_items() {
            if let Some(row) = rows.item(i).and_downcast::<TaskRow>() {
                if row.task().id() == id {
                    return Some(row);
                }
            }
        }
        None
    }

    pub fn activate_task_action(&self, name: &str, task: &Task) {
        let porject_id = self.section().project();
        self.transient_for()
            .unwrap()
            .activate_action(
                name,
                Some(&glib::Variant::from((
                    task.to_variant(),
                    ActionScope::Project(porject_id).to_variant(),
                ))),
            )
            .unwrap();
    }

    pub fn add_delete_toast(&self, task: &Task, toast: adw::Toast) {
        toast.connect_button_clicked(
            glib::clone!(@weak self as obj, @weak task => move |_toast| {
                if let Some(row) = obj.row_by_id(task.id()) {
                    row.task().set_suspended(false);
                    row.changed();
                }

                let main_win = obj.transient_for().and_downcast::<IPlanWindow>().unwrap();
                main_win
                    .activate_action(
                        "task.changed",
                        Some(&glib::Variant::from((
                            task.to_variant(),
                            ActionScope::DeleteToast.to_variant(),
                        ))),
                    )
                    .unwrap();
            }),
        );
    }

    #[template_callback]
    fn handle_tasks_box_row_activated(&self, row: gtk::ListBoxRow, _tasks_box: gtk::ListBox) {
        let obj = self.root().and_downcast::<gtk::Window>().unwrap();
        let row = row.downcast::<TaskRow>().unwrap();
        let modal = TaskWindow::new(&obj.application().unwrap(), &obj, row.task());
        modal.present();
        modal.connect_closure(
            "window-closed",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, task: Task| {
                let row = row.unwrap();
                if !task.done() {
                    obj.imp().tasks_box.remove(&row);
                    obj.emit_by_name::<()>("task-undo", &[&task]);
                } else {
                    row.reset(task);
                    row.changed();
                }
                let main_window = obj.transient_for().unwrap();
                main_window.activate_action("project.update", None).expect("Failed to send project.update signal");
            }
        ));
        modal.connect_closure(
            "task-changed",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, changed_task: Task| {
                let row = row.unwrap();
                let task = row.task();
                let task_id = task.id();
                if task_id == changed_task.id() {
                    row.reset(changed_task);
                } else if task_id == changed_task.parent() {
                    row.reset_subtasks();
                }
                obj.activate_task_action("task.changed", &task);
            }),
        );
        modal.connect_closure(
            "task-duration-changed",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, task: Task| {
                let row = row.unwrap();
                row.refresh_timer();
                obj.activate_task_action("task.duration-changed", &task);
            }),
        );
    }
}
