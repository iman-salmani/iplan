use gettextrs::gettext;
use gtk::{gdk, glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::db::models::{Record, Task};
use crate::db::operations::{create_record, delete_task, update_record, update_task};
use crate::views::project::{ProjectDoneTasksWindow, TaskWindow};
use crate::views::IPlanWindow;

#[derive(Default, PartialEq, Clone, Copy)]
pub enum TimerStatus {
    On,
    #[default]
    Off,
    Cancel,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/task_row.ui")]
    #[properties(wrapper_type=super::TaskRow)]
    pub struct TaskRow {
        #[property(get, set = Self::set_task)]
        pub task: RefCell<Task>,
        #[property(get, set)]
        pub moving_out: Cell<bool>,
        #[template_child]
        pub row_box: TemplateChild<gtk::Box>,
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
                glib::clone!(@weak obj => move |_controller, _keyval, keycode, _state| {
                    if keycode == 9 {   // Escape key
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
        @extends gtk::Widget, gtk::ListBoxRow,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl TaskRow {
    pub fn new(task: Task) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.reset(task);
        obj
    }

    pub fn reset(&self, task: Task) {
        let imp = self.imp();
        imp.name_entry_buffer.set_text(task.name());
        self.set_task(task);
        self.reset_timer();
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
            if duration != 0 {
                imp.timer_button_content
                    .set_label(&Record::duration_display(duration));
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
                    .and_downcast::<Self>()
                    .unwrap();
                let imp = obj.imp();
                let task = obj.task();
                task.set_done(active);
                update_task(&task).expect("Failed to update task");
                if active {
                    imp.timer_status.set(TimerStatus::Off);
                    obj.move_timer_button(false);
                }
                obj.activate_action("task.check", Some(&obj.index().to_variant()))
                    .expect("Failed to activate task.check action");
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
        let row_box: &gtk::Box = imp.row_box.as_ref();
        let options_button: &gtk::MenuButton = imp.options_button.as_ref();
        if indicate {
            button.unparent();
            options_button.popdown();
            button.remove_css_class("flat");
            button.insert_before(row_box, Some(options_button));
        } else {
            button.unparent();
            button.add_css_class("flat");
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
            self.move_timer_button(false);
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
                    delete_task(task.id(), task.list(), task.position())
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
                    .downcast::<ProjectDoneTasksWindow>()
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
        self.set_moving_out(false);
        self.changed();
        false
    }
}
