use adw::prelude::*;
use gettextrs::gettext;
use glib::Properties;
use gtk::{glib, subclass::prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::models::{Record, Task};
use crate::db::operations::read_tasks;
use crate::views::task::{TaskRow, TaskWindow, TasksBox, TasksBoxWrapper};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar/day_view.ui")]
    #[properties(wrapper_type=super::DayView)]
    pub struct DayView {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[property(get, set)]
        pub duration: Cell<i64>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub duration_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tasks_box: TemplateChild<TasksBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DayView {
        const NAME: &'static str = "DayView";
        type Type = super::DayView;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action(
                "task.check",
                Some(&Task::static_variant_type_string()),
                move |_, _, _| {},
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                duration: Cell::new(0),
                name: TemplateChild::default(),
                duration_label: TemplateChild::default(),
                tasks_box: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for DayView {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_bindings();
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
    impl WidgetImpl for DayView {}
    impl BoxImpl for DayView {}
}

glib::wrapper! {
    pub struct DayView(ObjectSubclass<imp::DayView>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl DayView {
    pub fn new(datetime: glib::DateTime) -> Self {
        let obj: DayView = glib::Object::new::<Self>();
        let imp = obj.imp();
        let end = datetime.add_days(1).unwrap().to_unix();

        let now = glib::DateTime::now_local().unwrap();
        if now.ymd() == datetime.ymd() {
            let name_format = format!("%e %b, {}", gettext("Today"));
            imp.name
                .set_label(&datetime.format(&name_format).unwrap().replace(" ", ""));
        } else {
            imp.name
                .set_label(&datetime.format("%e %b, %A").unwrap().replace(" ", ""));
        }

        let tasks = read_tasks(None, None, None, None, Some((datetime.to_unix(), end)))
            .expect("Failed to read tasks");
        let mut duration = 0;
        imp.tasks_box.set_scrollable(false);
        imp.tasks_box
            .set_items_wrapper(TasksBoxWrapper::Date(datetime.to_unix()));
        if tasks.is_empty() {
            imp.name.add_css_class("dim-label");
        } else {
            for task in tasks {
                duration += task.duration();
                imp.tasks_box.add_task(task);
            }
        }
        obj.set_duration(duration);

        obj.set_datetime(datetime);
        obj
    }

    pub fn add_row(&self, row: TaskRow) {
        let imp = self.imp();
        imp.tasks_box.set_visible(true);
        imp.tasks_box.add_item(&row);
        imp.name.remove_css_class("dim-label");
        self.set_duration(self.duration() + row.task().duration());
    }

    pub fn remove_row(&self, row: &TaskRow) {
        let imp = self.imp();
        imp.tasks_box.remove_item(&row);
        if imp.tasks_box.item_by_index(0).is_none() {
            imp.name.add_css_class("dim-label");
        }
    }

    fn add_bindings(&self) {
        self.bind_property::<gtk::Label>("duration", &self.imp().duration_label.get(), "label")
            .transform_to(|binding, duration: i64| {
                let duration_label = binding.target().unwrap();
                if duration == 0 {
                    duration_label.set_property("visible", false);
                    Some(String::new())
                } else {
                    duration_label.set_property("visible", true);
                    Some(Record::duration_display(duration))
                }
            })
            .build();
    }

    #[template_callback]
    fn task_activated(&self, row: TaskRow, _: gtk::ListBox) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = TaskWindow::new(&win.application().unwrap(), &win, row.task());
        modal.present();
        row.cancel_timer();
        modal.connect_closure(
            "window-closed",
            true,
            glib::closure_local!(@watch row => move |_win: TaskWindow, _: Task| {
                row.reset_timer();
            }),
        );
        modal.connect_closure(
            "task-changed",
            true,
            glib::closure_local!(@watch row => move |_win: TaskWindow, changed_task: Task| {
                row.activate_action("task.changed", Some(&changed_task.to_variant())).unwrap();
            }),
        );
        modal.connect_closure(
            "task-duration-changed",
            true,
            glib::closure_local!(@watch row => move |_win: TaskWindow, task_id: i64| {
                row.refresh_timer();
                row.activate_action("task.duration-changed", Some(&task_id.to_variant())).unwrap();
            }),
        );
    }
}
