use adw::traits::ExpanderRowExt;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::time::{SystemTime, UNIX_EPOCH};
use std::unimplemented;

use crate::db::models::{Record, Reminder, Task};
use crate::db::operations::{
    create_task, new_subtask_position, read_records, read_reminder, read_reminders, read_tasks,
    update_task,
};
use crate::views::record::{RecordRow, RecordWindow};
use crate::views::reminder::{ReminderRow, ReminderWindow};
use crate::views::snippets::DateRow;
use crate::views::task::{TaskRow, TasksBox, TasksBoxWrapper};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/task/task_page.ui")]
    pub struct TaskPage {
        #[template_child]
        pub task_row: TemplateChild<TaskRow>,
        #[template_child]
        pub reminders_expander_row: TemplateChild<adw::ExpanderRow>,
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
        pub subtasks_box: TemplateChild<TasksBox>,
        #[template_child]
        pub records_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub date_row: TemplateChild<DateRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskPage {
        const NAME: &'static str = "TaskPage";
        type Type = super::TaskPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.duration-update", None, move |obj, _, _value| {
                obj.imp().task_row.refresh_timer();
            });
            klass.install_action("project.update", None, move |obj, _, _value| {
                let task = obj.task();
                let imp = obj.imp();
                let mut records = read_records(task.id(), false, None, None).unwrap();
                imp.task_row.refresh_timer();
                if imp.records_box.observe_children().n_items() - 1 != records.len() as u32 {
                    records.sort_by_key(|record| record.id());
                    let row = RecordRow::new(records.last().unwrap().to_owned());
                    imp.records_box.append(&row);
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskPage {}
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
        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();

        let task_id = task.id();
        let task_description = task.description();
        let task_project = task.project();
        let task_date = task.date();
        imp.task_row.reset(task);

        imp.date_row.set_clear_option(true);
        let date = task_date;
        if date != 0 {
            imp.date_row.set_datetime_from_unix(date);
        }

        let reminders = read_reminders(Some(task_id)).expect("Failed to read reminders");
        for reminder in reminders {
            let row = obj.new_reminder_row(reminder);
            imp.reminders_expander_row.add_row(&row);
        }

        let task_description = task_description;
        imp.description_expander_row
            .set_subtitle(&obj.description_display(&task_description));
        imp.description_buffer.set_text(&task_description);

        imp.subtasks_box.set_scrollable(false);
        let tasks = read_tasks(None, None, None, Some(task_id), None).unwrap();
        imp.subtasks_box
            .set_items_wrapper(TasksBoxWrapper::Task(task_id, task_project));
        imp.subtasks_box.add_tasks(tasks);

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

        let records = read_records(task_id, false, None, None).expect("Failed to read records");
        for record in records {
            let row = RecordRow::new(record);
            imp.records_box.append(&row);
        }

        obj
    }

    pub fn task(&self) -> Task {
        self.imp().task_row.task()
    }

    pub fn add_reminder(&self, reminder_id: i64) {
        let imp = self.imp();
        let reminder = read_reminder(reminder_id).expect("Failed to read record");
        let row = self.new_reminder_row(reminder);
        imp.reminders_expander_row.add_row(&row);
        self.activate_action("task.changed", Some(&self.task().to_variant()))
            .unwrap();
    }

    fn new_reminder_row(&self, reminder: Reminder) -> ReminderRow {
        let row = ReminderRow::new(reminder);
        row.connect_closure(
            "changed",
            true,
            glib::closure_local!(@watch self as obj => move |_: ReminderRow, _: Reminder| {
                obj.activate_action("task.changed", Some(&obj.task().to_variant())).unwrap();
            }),
        );
        row.connect_closure(
            "removed",
            true,
            glib::closure_local!(@watch self as obj => move |_: ReminderRow, _: i64| {
                obj.activate_action("task.changed", Some(&obj.task().to_variant())).unwrap();
            }),
        );
        row
    }

    fn description_display(&self, text: &str) -> String {
        if let Some(first_line) = text.lines().next() {
            return String::from(first_line);
        }
        String::from("")
    }

    #[template_callback]
    fn handle_task_date_changed(&self, datetime: glib::DateTime, _: DateRow) {
        let task = self.task();
        task.set_date(datetime.to_unix());
        update_task(&task).expect("Failed to change update task");
        self.activate_action("task.changed", Some(&task.to_variant()))
            .unwrap();
    }

    #[template_callback]
    fn handle_new_reminder_clicked(&self, _: gtk::Button) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let reminder = Reminder::new(0, now as i64, false, self.task().id(), 2);
        let modal = ReminderWindow::new(&win.application().unwrap(), &win, reminder, false);
        modal.present();
    }

    #[template_callback]
    fn handle_description_buffer_changed(&self, buffer: gtk::TextBuffer) {
        let imp = self.imp();
        let task = self.task();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        if task.description() != text {
            imp.description_expander_row
                .set_subtitle(&self.description_display(&text));
            task.set_description(text);
            update_task(&task).unwrap();
            self.activate_action("task.changed", Some(&task.to_variant()))
                .unwrap();
        }
    }

    #[template_callback]
    fn handle_lists_menu_row_activated(&self, row: gtk::ListBoxRow, _lists_box: gtk::ListBox) {
        let imp = self.imp();
        imp.lists_popover.popdown();
        let label = row.child().and_downcast::<gtk::Label>().unwrap();
        match row.index() {
            // Subtasks
            0 => {
                imp.new_subtask_button.set_visible(true);
                imp.subtasks_box.set_visible(true);
                imp.lists_menu_button.set_label(&label.label());
            }
            // Records
            1 => {
                imp.new_subtask_button.set_visible(false);
                imp.subtasks_box.set_visible(false);
                imp.lists_menu_button.set_label(&label.label());
            }
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_new_record_button_clicked(&self, _button: gtk::Button) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let record = Record::new(0, now as i64, 0, self.task().id());
        let modal = RecordWindow::new(&win.application().unwrap(), &win, record, false);
        modal.present();
        modal.connect_closure(
            "record-created",
            true,
            glib::closure_local!(@watch self as obj => move |_win: RecordWindow, record: Record| {
                let imp = obj.imp();
                imp.task_row.refresh_timer();
                let row = RecordRow::new(record);
                imp.records_box.append(&row);
                }
            ),
        );
    }

    #[template_callback]
    fn handle_new_subtask_button_clicked(&self, _button: gtk::Button) {
        let task = self.task();
        let task_id = task.id();
        let subtask = Task::new(&[
            ("project", &task.project()),
            ("parent", &task_id),
            ("position", &new_subtask_position(task_id)),
        ]);
        let subtask = create_task(subtask).unwrap();
        self.activate_action("task.changed", Some(&subtask.to_variant()))
            .unwrap();
        self.imp().subtasks_box.add_fresh_task(subtask);
    }

    #[template_callback]
    fn subtask_activated(&self, subtask_row: TaskRow, _tasks_box: gtk::ListBox) {
        subtask_row.cancel_timer();
        self.activate_action("subtask.open", Some(&subtask_row.task().id().to_variant()))
            .expect("Failed to send subtask.open action");
    }
}
