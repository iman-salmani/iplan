use gettextrs::gettext;
use gtk::glib::{self, Properties};
use gtk::{gdk, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::db::models::{Record, Task};
use crate::db::operations::{
    create_record, delete_task, read_project, read_reminders, read_task, read_tasks, update_record,
    update_task,
};
use crate::views::snippets::MenuItem;
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
        pub backup_task_name: RefCell<String>,
        #[property(get, set)]
        pub moving_out: Cell<bool>,
        #[property(get, set)]
        pub compact: Cell<bool>,
        #[property(get, set)]
        pub lazy: Cell<bool>,
        #[property(get, set)]
        pub visible_project_label: Cell<bool>,
        #[property(get, set)]
        pub draggable: Cell<bool>,
        pub drag_backup: Cell<Option<DragBackup>>,
        #[property(get, set)]
        pub hide_move_arrows: Cell<bool>,
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
        pub timer_status: Cell<TimerStatus>,
        #[template_child]
        pub timer_button: TemplateChild<MenuItem>,
        #[template_child]
        pub timer_separator: TemplateChild<gtk::Separator>,
        #[template_child]
        pub options_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub options_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub options_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub move_up_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub move_down_button: TemplateChild<gtk::Button>,
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
        #[template_child]
        pub project_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TaskRow {
        const NAME: &'static str = "TaskRow";
        type Type = super::TaskRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            MenuItem::ensure_type();
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
                        obj.cancel_edit_name();
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
    pub fn new(task: Task, compact: bool, visible_project_label: bool) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_compact(compact);
        obj.set_visible_project_label(visible_project_label);
        obj.reset(task);
        obj.reset_timer();
        obj.set_draggable(true);
        obj
    }

    pub fn new_lazy(task: &Task, visible_project_label: bool) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_visible_project_label(visible_project_label);
        obj.set_task(task);
        obj.set_lazy(true);
        obj.set_draggable(true);
        obj.connect_lazy_notify(|obj| {
            let task = obj.task();
            obj.reset(task);
            obj.reset_timer();
        });
        obj
    }

    pub fn reset(&self, task: Task) {
        let imp = self.imp();

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
                imp.description.set_tooltip_text(Some(task_description));
                imp.body.set_visible(true);
            }

            if let Some(datetime) = task.date_datetime() {
                imp.date_indicator.set_label(&Task::date_display(&datetime));
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

            if self.visible_project_label() {
                let mut project_id = task.project();
                let mut parent = task.parent();
                loop {
                    if project_id != 0 {
                        let project = read_project(project_id).unwrap();
                        imp.project_label.set_label(&project.name());
                        break;
                    } else if parent == 0 {
                        imp.project_label.set_label(&gettext("Inbox"));
                        break;
                    } else {
                        let task = read_task(parent).unwrap();
                        project_id = task.project();
                        parent = task.parent();
                    }
                }
                imp.project_label.set_visible(true);
            } else {
                imp.project_label.set_visible(false);
            }

            if !imp.date_indicator.get_visible()
                && !imp.reminders_indicator.get_visible()
                && !imp.project_label.get_visible()
            {
                imp.footer.set_visible(false);
            } else {
                imp.footer.set_visible(true);
            }
        }

        let task_name = task.name();
        self.set_task(task);
        imp.name_entry.buffer().set_text(&task_name);
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

        let subtasks = read_tasks(None, None, None, Some(task.id()), None, false).unwrap();
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
        self.refresh_timer(); // FIXME: restore previous label
    }

    pub fn reset_timer(&self) {
        let imp = self.imp();
        let task = self.task();
        imp.timer_status.set(TimerStatus::Cancel); // FIXME: Check for removing this
        if let Some(record) = task.incomplete_record() {
            record.set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
            self.start_timer(record);
        } else {
            self.refresh_timer();
        }
    }

    pub fn refresh_timer(&self) {
        let imp = self.imp();

        if imp.timer_status.get() == TimerStatus::On {
            return;
        }

        let duration = self.task().duration();
        if duration == 0 {
            imp.timer_button.set_label(gettext("Start _Timer"));
            self.move_timer_button(false);
        } else {
            imp.timer_button
                .set_label(Record::duration_display(duration));
            self.move_timer_button(true);
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
            parent.invalidate_filter();
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
                obj.activate_action("task.changed", Some(&task.to_variant()))
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

        self.connect_hide_move_arrows_notify(|obj| {
            let imp = obj.imp();
            let visible = !obj.hide_move_arrows();
            imp.move_up_button.set_visible(visible);
            imp.move_down_button.set_visible(visible);
            imp.move_down_button
                .next_sibling()
                .and_downcast::<gtk::Separator>()
                .unwrap()
                .set_visible(visible);
        });

        task.bind_property("id", &imp.move_up_button.get(), "action-target")
            .transform_to(|_, id: i64| Some(id.to_variant()))
            .build();
        task.bind_property("id", &imp.move_down_button.get(), "action-target")
            .transform_to(|_, id: i64| Some(id.to_variant()))
            .build();
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
        self.set_backup_task_name(self.task().name());
    }

    #[template_callback]
    fn handle_name_entry_changed(&self, entry: gtk::Entry) {
        let task = self.task();
        let text = entry.text();

        if text == task.name() {
            return;
        }

        task.set_name(text);
        self.activate_action("task.changed", Some(&task.to_variant()))
            .unwrap();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, _entry: gtk::Entry) {
        self.imp().name_button.set_visible(true);
    }

    #[template_callback]
    fn handle_name_entry_icon_press(&self, _: gtk::EntryIconPosition) {
        self.cancel_edit_name();
    }

    fn cancel_edit_name(&self) {
        let imp = self.imp();
        let name = self.backup_task_name();
        imp.name_entry.buffer().set_text(&name);
        imp.name_button.set_visible(true);
    }

    fn move_timer_button(&self, indicate: bool) {
        let imp = self.imp();
        let button: &MenuItem = imp.timer_button.as_ref();
        let options_box: &gtk::Box = imp.options_box.as_ref();
        if indicate {
            let options_button: &gtk::MenuButton = imp.options_button.as_ref();
            let header: &gtk::Box = imp.header.as_ref();
            button.unparent();
            options_button.popdown();
            button.insert_before(header, Some(options_button));
            imp.timer_separator.set_visible(false);
        } else {
            button.unparent();
            options_box.prepend(button);
            imp.timer_separator.set_visible(true);
        }
    }

    fn start_timer(&self, record: Record) {
        let imp = self.imp();
        let button: &MenuItem = imp.timer_button.as_ref();
        imp.timer_status.set(TimerStatus::On);
        button.add_css_class("destructive-action");
        button.remove_css_class("flat");
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
                        button.set_label(
                            Record::duration_display(duration as i64)
                        );
                        glib::Continue(true)
                    },
                    TimerStatus::Off => {
                        button.remove_css_class("destructive-action");
                        button.add_css_class("flat");
                        record.set_duration(glib::DateTime::now_local().unwrap().to_unix() - record.start());
                        update_record(&record).expect("Failed to update record");
                        let task = obj.task();
                        imp.timer_button.set_label(task.duration_display());
                        if obj.parent().is_some() {
                            obj.activate_action("task.duration-changed", Some(&task.to_variant())).unwrap();
                        }
                        glib::Continue(false)
                    },
                    TimerStatus::Cancel => {
                        button.remove_css_class("destructive-action");
                        button.add_css_class("flat");
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
        if let Some((i, _)) = toast_name.char_indices().nth(14) {
            toast_name.truncate(i);
            toast_name.push_str("...");
        }
        let toast = adw::Toast::builder()
            .title(gettext("\"{}\" is going to delete").replace("{}", &toast_name))
            .button_label(gettext("Undo"))
            .build();

        toast.connect_button_clicked(glib::clone!(@weak self as obj => move |_toast| {
            let task = obj.task();
            task.set_suspended(false);
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
                    delete_task(&task).unwrap();
                }
            }
        ));

        task.set_suspended(true);
        update_task(&task).expect("Failed to update task");
        self.activate_action("task.changed", Some(&task.to_variant()))
            .unwrap();
        self.changed();

        let window = self.root().unwrap();
        match window.widget_name().as_str() {
            "IPlanWindow" => {
                window
                    .downcast::<IPlanWindow>()
                    .unwrap()
                    .add_delete_toast(&task, toast);
            }
            "TasksDoneWindow" => {
                window
                    .downcast::<TasksDoneWindow>()
                    .unwrap()
                    .add_delete_toast(&task, toast);
            }
            "TaskWindow" => {
                window
                    .downcast::<TaskWindow>()
                    .unwrap()
                    .add_delete_toast(&task, toast);
            }
            _ => unimplemented!(),
        }
    }

    #[template_callback]
    fn handle_drag_prepare(&self, _x: f64, _y: f64) -> Option<gdk::ContentProvider> {
        if !self.draggable() {
            None
        } else if self.imp().name_entry.get_visible() {
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
