use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{glib, glib::Properties};
use std::cell::RefCell;

use crate::db::models::{Section, Task};
use crate::db::operations::{read_task, read_tasks};
use crate::views::task::{TaskRow, TaskWindow};
use crate::views::IPlanWindow;

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
            klass.install_action("task.check", Some("i"), move |obj, _, value| {
                let imp = obj.imp();
                let index = value.unwrap().get().unwrap();
                let upper_row = imp.tasks_box.row_at_index(index - 1);
                let row = imp.tasks_box.row_at_index(index).unwrap();
                if let Some(upper_row) = upper_row {
                    upper_row.grab_focus();
                }
                imp.tasks_box.remove(&row);
                obj.transient_for()
                    .unwrap()
                    .activate_action("project.open", None) // TODO: just add task to section
                    .unwrap();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TasksDoneWindow {
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

    #[template_callback]
    fn handle_tasks_box_row_activated(&self, row: gtk::ListBoxRow, _tasks_box: gtk::ListBox) {
        let obj = self.root().and_downcast::<gtk::Window>().unwrap();
        let row = row.downcast::<TaskRow>().unwrap();
        let modal = TaskWindow::new(&obj.application().unwrap(), &obj, row.task());
        modal.present();
        modal.connect_closure(
            "task-window-close",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, task: Task| {
                let row = row.unwrap();
                let main_window = obj.transient_for().unwrap();
                if !task.done() {
                    obj.imp().tasks_box.remove(&row);
                    main_window.activate_action("project.open", None) // TODO: just add task to section (consider the task duration could be changed)
                        .expect("Failed to activate project.open action");
                } else {
                    row.reset(task);
                    row.changed();
                    main_window.activate_action("project.update", None).expect("Failed to send project.update signal");
                }
            }
        ));
    }
}
