use gettextrs::gettext;
use gtk::{gdk, glib, glib::once_cell::sync::Lazy, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::Duration;

use crate::db::models::{Record, Task};
use crate::db::operations::{create_record, delete_task, read_records, update_record, update_task};
use crate::views::{project::ProjectDoneTasksWindow, project::TaskWindow, IPlanWindow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_row.ui")]
    pub struct TaskRow {
        pub task: RefCell<Task>,
        pub moving_out: Cell<bool>,
        #[template_child]
        pub checkbox: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub name_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub timer_toggle_button: TemplateChild<gtk::ToggleButton>,
        pub timer_toggle_button_handler_id: RefCell<Option<gtk::glib::SignalHandlerId>>,
        #[template_child]
        pub timer_button_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub options_popover: TemplateChild<gtk::Popover>,
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
    impl WidgetImpl for TaskRow {}
    impl ListBoxRowImpl for TaskRow {}
}

glib::wrapper! {
    pub struct TaskRow(ObjectSubclass<imp::TaskRow>)
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl TaskRow {
    pub fn new(task: Task) -> Self {
        glib::Object::builder().property("task", task).build()
    }

    pub fn init_widgets(&self) {
        let imp = self.imp();
        let task = self.task();
        let task_name = task.name();

        imp.checkbox.set_active(task.done());
        imp.name_button
            .child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .set_text(&task_name);
        imp.name_button.set_tooltip_text(Some(&task_name));
        imp.name_entry.buffer().set_text(&task_name);
        let name_entry_controller = gtk::EventControllerKey::new();
        name_entry_controller.connect_key_released(glib::clone!(
        @weak self as obj =>
        move |_controller, _keyval, keycode, _state| {
            if keycode == 9 {   // Escape
                let imp = obj.imp();
                imp.name_button.set_visible(true);
                imp.name_entry.buffer().set_text(&obj.task().name());
            }
        }));
        imp.name_entry.add_controller(&name_entry_controller);
        if let Some(duration) = task.duration() {
            imp.timer_button_content
                .set_label(&Record::duration_display(duration));
        }
        if task.done() {
            imp.timer_toggle_button.set_sensitive(false);
        } else {
            let handler_id = imp.timer_toggle_button.connect_toggled(glib::clone!(
                @weak self as obj =>
                move |button| obj.handle_timer_toggle_button_toggled(button)));
            imp.timer_toggle_button_handler_id.replace(Some(handler_id));
            // Starting timer if have not finished record
            let records =
                read_records(task.id(), true, None, None).expect("Failed to read records");
            if !records.is_empty() {
                imp.timer_toggle_button.set_active(true)
            }
        }
    }

    pub fn task(&self) -> Task {
        self.property("task")
    }

    #[template_callback]
    fn handle_done_check_button_toggled(&self, button: gtk::CheckButton) {
        let imp = self.imp();
        let task = self.task();
        let is_active = button.is_active();

        if task.done() != is_active {
            // This happens in fetch done tasks
            task.set_property("done", is_active);
            self.set_property("task", task);
            update_task(self.task()).expect("Failed to update task");

            if is_active {
                imp.timer_toggle_button.set_active(false);
                imp.timer_toggle_button.set_sensitive(false);
            } else {
                imp.timer_toggle_button.set_sensitive(true);
                if imp.timer_toggle_button_handler_id.borrow().is_none() {
                    let handler_id = imp.timer_toggle_button.connect_toggled(glib::clone!(
                        @weak self as obj =>
                        move |button| obj.handle_timer_toggle_button_toggled(button)));
                    imp.timer_toggle_button_handler_id.replace(Some(handler_id));
                }
            }

            self.activate_action("task.check", Some(&self.index().to_variant()))
                .expect("Failed to activate task.check action");
        }
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let name = entry.buffer().text();
        let task = self.task();
        let imp = self.imp();
        imp.name_button
            .child()
            .unwrap()
            .downcast::<gtk::Label>()
            .unwrap()
            .set_text(&name);
        imp.name_button.set_visible(true);
        imp.name_button.set_tooltip_text(Some(&name));
        task.set_property("name", name);
        update_task(task).expect("Failed to update task");
    }

    #[template_callback]
    fn handle_name_entry_icon_press(&self, _: gtk::EntryIconPosition) {
        let imp = self.imp();
        imp.name_button.set_visible(true);
        imp.name_entry.buffer().set_text(&self.task().name());
    }

    fn handle_timer_toggle_button_toggled(&self, button: &gtk::ToggleButton) {
        let task = self.task();
        let records = read_records(task.id(), true, None, None).expect("Failed to read records");
        let record = if records.is_empty() {
            create_record(glib::DateTime::now_local().unwrap().to_unix(), task.id(), 0)
                .expect("Failed to create record")
        } else {
            let record = records.get(0).unwrap().clone();
            record.set_property(
                "duration",
                glib::DateTime::now_local().unwrap().to_unix() - record.start(),
            );
            record
        };

        if button.is_active() {
            button.add_css_class("destructive-action");

            let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let mut duration = record.duration();
            thread::spawn(move || loop {
                if let Err(_) = tx.send(duration.to_string()) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
                duration += 1;
            });
            rx.attach(
                None,
                glib::clone!(
                    @weak button => @default-return glib::Continue(false),
                    move |text| {
                        if button.is_active() {
                            let button_content = button.child()
                                .and_downcast::<adw::ButtonContent>()
                                .unwrap();
                            button_content.set_label(
                                &Record::duration_display(text.parse::<i64>().unwrap())
                            );
                            glib::Continue(true)
                        } else {
                            glib::Continue(false)
                        }
                    }
                ),
            );
        } else {
            button.remove_css_class("destructive-action");
            record.set_property(
                "duration",
                glib::DateTime::now_local().unwrap().to_unix() - record.start(),
            );
            update_record(&record).expect("Failed to update record");
            self.imp()
                .timer_button_content
                .set_label(&Record::duration_display(
                    task.duration()
                        .expect("Task duration cannot be 0 at this point"),
                ));
            self.activate_action("project.update", None)
                .expect("Failed to send project.update");
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
        let toast_name = gettext("\"{}\" is going to delete");
        let toast = adw::Toast::builder()
            .title(&toast_name.replace("{}", &toast_name))
            .button_label(&gettext("Undo"))
            .build();

        toast.connect_button_clicked(glib::clone!(
        @weak self as obj =>
        move |_toast| {
            let task = obj.task();
            task.set_property("suspended", false);
            update_task(task).expect("Failed to update task");
            if obj.parent().is_some() {
                obj.changed();
                obj.grab_focus();
            }
        }));
        toast.connect_dismissed(glib::clone!(
            @strong self as obj =>
            move |_toast| {
                let task = obj.task();
                if task.suspended() {    // Checking Undo button
                    delete_task(task.id(), task.list(), task.position())
                        .expect("Failed to delete task");
                    if let Some(tasks_box) = obj.parent() {    // check for project active changed
                        let tasks_box = tasks_box.downcast::<gtk::ListBox>().unwrap();
                        for i in 0..obj.index() {
                            let row = tasks_box.row_at_index(i).unwrap();
                            let row_task: Task = row.property("task");
                            row_task.set_property("position", row_task.position() - 1);
                        }
                        let upper_row = obj.parent()
                            .and_downcast::<gtk::ListBox>()
                            .unwrap()
                            .row_at_index(obj.index() - 1);
                        if let Some(upper_row) = upper_row {upper_row.grab_focus();}
                        tasks_box.remove(&obj);
                    }
                }
            }
        ));
        let win_name = self.root().unwrap().widget_name();
        if win_name == "IPlanWindow" {
            let window = self.root().and_downcast::<IPlanWindow>().unwrap();
            window.imp().toast_overlay.add_toast(&toast);
        } else if win_name == "ProjectDoneTasksWindow" {
            let window = self
                .root()
                .and_downcast::<ProjectDoneTasksWindow>()
                .unwrap();
            window.imp().toast_overlay.add_toast(&toast);
        } else if win_name == "TaskWindow" {
            let window = self.root().and_downcast::<TaskWindow>().unwrap();
            window.imp().toast_overlay.add_toast(&toast);
        }
        task.set_property("suspended", true);
        self.set_property("task", &task);
        update_task(task).expect("Failed to update task");
        let upper_row = self
            .parent()
            .and_downcast::<gtk::ListBox>()
            .unwrap()
            .row_at_index(self.index() - 1);
        if let Some(upper_row) = upper_row {
            upper_row.grab_focus();
        }
        self.changed();
    }

    #[template_callback]
    fn handle_drag_prepare(&self, _x: f64, _y: f64) -> Option<gdk::ContentProvider> {
        let name_entry = self.imp().name_entry.get();
        if WidgetExt::is_visible(&name_entry) {
            None
        } else {
            Some(gdk::ContentProvider::for_value(&self.to_value()))
        }
    }

    #[template_callback]
    fn handle_drag_begin(&self, drag: gdk::Drag) {
        self.parent()
            .and_downcast::<gtk::ListBox>()
            .unwrap()
            .drag_highlight_row(self);
        let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(&drag).downcast().unwrap();
        let label = gtk::Label::builder().label("").build();
        drag_icon.set_child(Some(&label));
        drag.set_hotspot(0, 0);
    }

    #[template_callback]
    fn handle_drag_cancel(&self, _drag: gdk::Drag) -> bool {
        self.imp().moving_out.set(false);
        self.changed();
        false
    }
}
