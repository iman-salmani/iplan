use adw::traits::ExpanderRowExt;
use gtk::{glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::RefCell;
use std::unimplemented;

use crate::db::models::{Record, Task};
use crate::db::operations::{create_task, read_record, read_records, read_tasks, update_task};
use crate::views::project::{RecordCreateWindow, RecordRow, TaskRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_page.ui")]
    pub struct TaskPage {
        pub task: RefCell<Task>,
        #[template_child]
        pub task_row: TemplateChild<TaskRow>,
        #[template_child]
        pub description_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub description_buffer: TemplateChild<gtk::TextBuffer>,
        #[template_child]
        pub lists_menu_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub lists_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub new_subtask_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub new_record_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub subtasks_page: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub subtasks_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub records_page: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub records_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskPage {
        const NAME: &'static str = "TaskPage";
        type Type = super::TaskPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, _value| {
                let imp = obj.imp();
                imp.subtasks_box.invalidate_sort();
            });
            klass.install_action("task.duration-update", None, move |obj, _, _value| {
                let imp = obj.imp();
                let task_row_imp = imp.task_row.imp();
                if let Some(duration) = imp.task_row.task().duration() {
                    if !task_row_imp.timer_toggle_button.is_active() {
                        task_row_imp
                            .timer_button_content
                            .set_label(&Record::duration_display(duration));
                    }
                }
            });
            klass.install_action("record.created", Some("x"), move |obj, _, value| {
                let record_id = value.unwrap().get::<i64>().unwrap();
                let imp = obj.imp();
                let task_row_imp = imp.task_row.imp();
                if !task_row_imp.timer_toggle_button.is_active() {
                    task_row_imp
                        .timer_button_content
                        .set_label(&Record::duration_display(
                            imp.task_row
                                .task()
                                .duration()
                                .expect("Task duration cant be 0 at this point"),
                        ));
                }
                let record = read_record(record_id).expect("Failed to read record");
                let row = RecordRow::new(record);
                imp.records_box.append(&row);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskPage {
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
    impl WidgetImpl for TaskPage {}
    impl BoxImpl for TaskPage {}
}

glib::wrapper! {
    pub struct TaskPage(ObjectSubclass<imp::TaskPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable, gtk::Orientable;
}

#[gtk::template_callbacks]
impl TaskPage {
    pub fn new(task: Task) -> Self {
        let page: Self = glib::Object::builder().build();
        let imp = page.imp();
        imp.task_row.set_property("task", task.clone());
        imp.task_row.init_widgets();
        let task_description = task.description();
        imp.description_expander_row
            .set_subtitle(&page.description_display(&task_description));
        imp.description_buffer.set_text(&task_description);
        imp.subtasks_box.set_sort_func(|row1, _row2| {
            if row1.property::<Task>("task").done() {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });
        imp.subtasks_box.set_filter_func(glib::clone!(
            @weak imp => @default-return false,
            move |row| {
                let row = row.downcast_ref::<TaskRow>().unwrap();
                !row.task().suspended()
        }));
        let tasks = read_tasks(task.project(), None, None, Some(task.id()))
            .expect("Failed to read subtasks");
        for task in tasks {
            let row = TaskRow::new(task);
            row.init_widgets();
            imp.subtasks_box.append(&row);
        }
        imp.records_box
            .set_sort_func(|row1: &gtk::ListBoxRow, row2| {
                let row1_start = row1.property::<Record>("record").start();
                let row2_start = row2.property::<Record>("record").start();

                if row1_start > row2_start {
                    gtk::Ordering::Smaller
                } else {
                    gtk::Ordering::Larger
                }
            });
        let records = read_records(task.id(), false, None, None).expect("Failed to read records");
        for record in records {
            let row = RecordRow::new(record);
            imp.records_box.append(&row);
        }
        imp.task.replace(task);
        page
    }

    pub fn task(&self) -> Task {
        self.property("task")
    }

    fn description_display(&self, text: &str) -> String {
        if let Some(first_line) = text.lines().next() {
            return String::from(first_line);
        }
        String::from("")
    }

    #[template_callback]
    fn handle_description_buffer_changed(&self, buffer: gtk::TextBuffer) {
        let imp = self.imp();
        let task = self.task();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        imp.description_expander_row
            .set_subtitle(&self.description_display(&text));
        task.set_property("description", text);
        update_task(task).expect("Failed to update task");
    }

    #[template_callback]
    fn handle_lists_menu_row_activated(&self, row: gtk::ListBoxRow, _lists_box: gtk::ListBox) {
        let imp = self.imp();
        imp.lists_popover.popdown();
        match row.index() {
            // Subtasks
            0 => {
                imp.new_subtask_button.set_visible(true);
                imp.subtasks_page.set_visible(true);
            }
            // Records
            1 => {
                imp.new_subtask_button.set_visible(false);
                imp.subtasks_page.set_visible(false);
            }
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_new_record_button_clicked(&self, _button: gtk::Button) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = RecordCreateWindow::new(&win.application().unwrap(), &win, self.task().id());
        modal.present();
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

    #[template_callback]
    fn handle_subtasks_box_row_activated(&self, row: gtk::ListBoxRow, _tasks_box: gtk::ListBox) {
        let row = row.downcast::<TaskRow>().unwrap();
        self.activate_action("subtask.open", Some(&row.task().id().to_variant()))
            .expect("Failed to send subtask.open action");
    }
}
