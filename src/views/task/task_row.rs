use gettextrs::gettext;
use gtk::{gdk, glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::db::models::{Record, Task};
use crate::db::operations::{
    create_record, delete_task, read_reminders, read_tasks, update_record, update_task,
};
use crate::views::task::{SubtaskRow, TaskWindow, TasksDoneWindow};
use crate::views::IPlanWindow;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum TimerStatus {
    On,
    #[default]
    Off,
    Cancel,
}

pub struct DragBackup {
    position: i32,
    section: i64,
    parent_task: i64,
    parent_widget: gtk::ListBox,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/task/task_row.ui")]
    #[properties(wrapper_type=super::TaskRow)]
    pub struct TaskRow {
        #[property(get, set = Self::set_task)]
        pub task: RefCell<Task>,
        #[property(get, set)]
        pub moving_out: Cell<bool>,
        #[property(get, set)]
        pub compact: Cell<bool>,
        #[property(get, set)]
        pub lazy: Cell<bool>,
        pub drag_backup: Cell<Option<DragBackup>>,
        #[template_child]
        pub row_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub header: TemplateChild<gtk::Box>,
        #[template_child]
        pub checkbox: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub name_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub name_entry_buffer: TemplateChild<gtk::EntryBuffer>,
        pub timer_status: Cell<TimerStatus>,
        #[template_child]
        pub timer_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub timer_button_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub options_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub options_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub options_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub body: TemplateChild<gtk::Box>,
        #[template_child]
        pub subtasks: TemplateChild<gtk::Box>,
        #[template_child]
        pub subtask_drop_target: TemplateChild<gtk::Widget>,
        #[template_child]
        pub footer: TemplateChild<gtk::Box>,
        #[template_child]
        pub date_indicator: TemplateChild<gtk::Label>,
        #[template_child]
        pub reminders_indicator: TemplateChild<gtk::Image>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskRow {
        const NAME: &'static str = "TaskRow";
        type Type = super::TaskRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TaskRow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            let imp = obj.imp();

            obj.add_bindings();

            // Cancel name entry on Escape key pressed
            let name_entry_controller = gtk::EventControllerKey::new();
            name_entry_controller.connect_key_released(
                glib::clone!(@weak obj => move |_controller, keyval, _keycode, _state| {
                    if keyval == gdk::Key::Escape {
                        let imp = obj.imp();
                        imp.name_button.set_visible(true);
                        imp.name_entry.buffer().set_text(obj.task().name());
                    }
                }),
            );
            imp.name_entry.add_controller(name_entry_controller);
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

    impl WidgetImpl for TaskRow {}
    impl ListBoxRowImpl for TaskRow {}

    impl TaskRow {
        fn set_task(&self, new_task: Task) {
            let old_task = self.task.borrow();
            for property in old_task.list_properties() {
                let property_name = property.name();
                old_task.set_property_from_value(property_name, &new_task.property(property_name));
            }
        }
    }
}

glib::wrapper! {
    pub struct TaskRow(ObjectSubclass<imp::TaskRow>)
        @extends glib::InitiallyUnowned, gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl TaskRow {
    pub fn new(task: Task, compact: bool) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_compact(compact);
        obj.reset(task);
        obj
    }

    pub fn new_lazy(task: &Task) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_task(task);
        obj.set_lazy(true);
        obj.connect_lazy_notify(|obj| {
            let task = obj.task();
            obj.reset(task);
        });
        obj
    }

    pub fn reset(&self, task: Task) {
        let imp = self.imp();
        imp.name_entry_buffer.set_text(task.name());

        if self.compact() {
            self.remove_css_class("card");
            self.set_margin_bottom(0);
            imp.body.set_visible(false);
            imp.subtasks.set_visible(false);
            imp.footer.set_visible(false);
        } else {
            let task_description = task.description();
            let task_description = task_description.trim();
            if task_description.is_empty() {
                imp.body.set_visible(false);
            } else {
                imp.description.set_label(task_description);
                imp.body.set_visible(true);
            }

            if let Some(datetime) = task.date_datetime() {
                imp.date_indicator
                    .set_label(&datetime.format("%A").unwrap());
                imp.date_indicator.set_visible(true);
            } else {
                imp.date_indicator.set_visible(false);
            }

            let reminders = read_reminders(Some(task.id())).unwrap();
            if reminders.is_empty() {
                imp.reminders_indicator.set_visible(false);
            } else {
                imp.reminders_indicator.set_visible(true);
            }

            if !imp.date_indicator.get_visible() && !imp.reminders_indicator.get_visible() {
                imp.footer.set_visible(false);
            } else {
                imp.footer.set_visible(true);
            }
        }

        self.set_task(task);
        self.reset_timer();
        if !self.compact() {
            self.reset_subtasks();
        }
    }

    pub fn reset_subtasks(&self) {
        let imp = self.imp();
        let task = self.task();

        loop {
            if let Some(subtask) = imp.subtasks.first_child() {
                imp.subtasks.remove(&subtask);
            } else {
                break;
            }
        }

        let subtasks = read_tasks(None, None, None, Some(task.id()), None).unwrap();
        if subtasks.is_empty() {
            imp.subtasks.set_visible(false);
        } else {
            imp.subtasks.set_visible(true);
            for subtask in subtasks {
                let subtask_row = SubtaskRow::new(subtask);
                imp.subtasks.append(&subtask_row);
            }
        }
    }

    pub fn add_subtask(&self, subtask: Task) {
        let imp = self.imp();
        imp.subtasks.set_visible(true);
        let subtask_row = SubtaskRow::new(subtask);
        imp.subtasks.prepend(&subtask_row);
    }

    pub fn cancel_timer(&self) {
        self.imp().timer_status.set(TimerStatus::Cancel);
        self.move_timer_button(false);
    }

    pub fn reset_timer(&self) {
        let imp = self.imp();
        let task = self.task();
        imp.timer_status.set(TimerStatus::Cancel); // FIXME: Check for removing this
        if let Some(record) = task.incomplete_record() {
            record.set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
            self.start_timer(record);
        } else {
            let duration = task.duration();
            if duration == 0 {
                imp.timer_button_content.set_label(&gettext("Start _timer"));
            } else {
                imp.timer_button_content
                    .set_label(&Record::duration_display(duration));
                self.move_timer_button(true);
            }
        }
    }

    pub fn refresh_timer(&self) {
        let imp = self.imp();
        if imp.timer_status.get() != TimerStatus::On {
            imp.timer_button_content
                .set_label(&self.task().duration_display());
        }
    }

    pub fn keep_after_dnd(&self) {
        self.set_moving_out(false);
        let drag_backup = self.imp().drag_backup.take().unwrap();
        let task = self.task();
        task.set_position(drag_backup.position);
        task.set_section(drag_backup.section);
        task.set_parent(drag_backup.parent_task);
        let parent = self.parent().and_downcast::<gtk::ListBox>().unwrap();
        if parent != drag_backup.parent_widget {
            parent.remove(self);
            drag_backup.parent_widget.append(self);
        }
        self.changed();
    }

    fn add_bindings(&self) {
        let imp = self.imp();
        let task = self.task();

        task.bind_property("done", &imp.checkbox.get(), "active")
            .transform_from(|binding, active: bool| {
                let checkbox = binding.target().and_downcast::<gtk::CheckButton>().unwrap();
                let obj = checkbox
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .and_downcast::<Self>()
                    .unwrap();
                let imp = obj.imp();
                let task = obj.task();
                task.set_done(active);
                update_task(&task).expect("Failed to update task");
                if active {
                    imp.timer_status.set(TimerStatus::Off);
                }
                obj.activate_action("task.check", Some(&obj.index().to_variant()))
                    .unwrap();
                Some(active)
            })
            .sync_create()
            .bidirectional()
            .build();

        task.bind_property("done", &imp.timer_button.get(), "sensitive")
            .sync_create()
            .invert_boolean()
            .build();
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let task = self.task();
        self.imp().name_button.set_visible(true);
        task.set_name(entry.buffer().text());
        update_task(&task).expect("Failed to update task");
    }

    #[template_callback]
    fn handle_name_entry_icon_press(&self, _: gtk::EntryIconPosition) {
        let imp = self.imp();
        imp.name_entry.buffer().set_text(self.task().name());
        imp.name_button.set_visible(true);
    }

    fn move_timer_button(&self, indicate: bool) {
        let imp = self.imp();
        let button: &gtk::Button = imp.timer_button.as_ref();
        let options_box: &gtk::Box = imp.options_box.as_ref();
        if indicate {
            let options_button: &gtk::MenuButton = imp.options_button.as_ref();
            let header: &gtk::Box = imp.header.as_ref();
            button.unparent();
            options_button.popdown();
            button.remove_css_class("flat");
            button.insert_before(header, Some(options_button));
        } else {
            button.add_css_class("flat");
            button.unparent();
            options_box.prepend(button);
        }
    }

    fn start_timer(&self, record: Record) {
        let imp = self.imp();
        let button: &gtk::Button = imp.timer_button.as_ref();
        imp.timer_status.set(TimerStatus::On);
        button.add_css_class("destructive-action");
        self.move_timer_button(true);

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let duration = Duration::from_secs(record.duration() as u64);
        let start = SystemTime::now().checked_sub(duration).unwrap();
        thread::spawn(move || loop {
            if tx.send(start.elapsed().unwrap().as_secs()).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.3));
        });
        rx.attach(None,glib::clone!(@weak button, @weak self as obj => @default-return glib::Continue(false),
            move |duration| {
                let imp = obj.imp();
                match imp.timer_status.get() {
                    TimerStatus::On => {
                        let button_content = button.child()
                            .and_downcast::<adw::ButtonContent>()
                            .unwrap();
                        button_content.set_label(
                            &Record::duration_display(duration as i64)
                        );
                        glib::Continue(true)
                    },
                    TimerStatus::Off => {
                        button.remove_css_class("destructive-action");
                        record.set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
                        update_record(&record).expect("Failed to update record");
                        imp.timer_button_content.set_label(&obj.task().duration_display());
                        if obj.parent().is_some() {
                            obj.activate_action("project.update", None)
                                .expect("Failed to send project.update");
                        }
                        glib::Continue(false)
                    },
                    TimerStatus::Cancel => {
                        button.remove_css_class("destructive-action");
                        glib::Continue(false)
                    }
                }
            }
            ),
        );
    }

    #[template_callback]
    fn handle_timer_button_clicked(&self, _button: &gtk::Button) {
        let task = self.task();
        let record = task.incomplete_record().unwrap_or_else(|| {
            create_record(glib::DateTime::now_local().unwrap().to_unix(), task.id(), 0)
                .expect("Failed to create record")
        });
        if self.imp().timer_status.get() != TimerStatus::On {
            self.start_timer(record);
        } else {
            self.imp().timer_status.set(TimerStatus::Off);
        }
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let task = self.task();
        let mut toast_name = task.name();
        if toast_name.chars().count() > 15 {
            toast_name.truncate(15);
            toast_name.push_str("...");
        }
        let toast = adw::Toast::builder()
            .title(gettext("\"{}\" is going to delete").replace("{}", &toast_name))
            .button_label(gettext("Undo"))
            .build();

        toast.connect_button_clicked(glib::clone!(@weak self as obj =>
            move |_toast| {
                let task = obj.task();
                task.set_property("suspended", false);
                update_task(&task).expect("Failed to update task");
                if obj.parent().is_some() {
                    obj.changed();
                    obj.grab_focus();
                }
        }));
        toast.connect_dismissed(glib::clone!(@strong self as obj =>
            move |_toast| {
                let task = obj.task();
                if task.suspended() {    // Checking Undo button
                    delete_task(task.id(), task.section(), task.position())
                        .expect("Failed to delete task");
                }
            }
        ));
        task.set_suspended(true);
        self.set_task(&task);
        update_task(&task).expect("Failed to update task");
        self.changed();
        let window = self.root().unwrap();
        match window.widget_name().as_str() {
            "IPlanWindow" => {
                window
                    .downcast::<IPlanWindow>()
                    .unwrap()
                    .imp()
                    .toast_overlay
                    .add_toast(toast);
            }
            "ProjectDoneTasksWindow" => {
                window
                    .downcast::<TasksDoneWindow>()
                    .unwrap()
                    .imp()
                    .toast_overlay
                    .add_toast(toast);
            }
            "TaskWindow" => {
                window
                    .downcast::<TaskWindow>()
                    .unwrap()
                    .add_toast(task, toast);
            }
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_drag_prepare(&self, _x: f64, _y: f64) -> Option<gdk::ContentProvider> {
        if self.imp().name_entry.get_visible() {
            None
        } else {
            Some(gdk::ContentProvider::for_value(&self.to_value()))
        }
    }

    #[template_callback]
    fn handle_drag_begin(&self, drag: gdk::Drag) {
        let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(&drag).downcast().unwrap();
        let label = gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/task/task_drag_icon.ui")
            .object::<gtk::Box>("task_drag_icon")
            .unwrap();
        label
            .last_child()
            .unwrap()
            .set_property("label", &self.task().name());
        drag_icon.set_child(Some(&label));
        self.add_css_class("dragged");
        drag.set_hotspot(0, 0);
        let task = self.task();
        self.imp().drag_backup.set(Some(DragBackup {
            position: task.position(),
            section: task.section(),
            parent_task: task.parent(),
            parent_widget: self.parent().and_downcast::<gtk::ListBox>().unwrap(),
        }))
    }

    #[template_callback]
    fn handle_drag_cancel(&self, _drag: gdk::Drag) -> bool {
        self.keep_after_dnd();
        false
    }

    #[template_callback]
    fn handle_drag_end(&self, _drag: gdk::Drag, _: bool) {
        self.remove_css_class("dragged");
    }
}
