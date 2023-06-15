use adw::prelude::*;
use gettextrs::gettext;
use glib::{once_cell::sync::Lazy, subclass::Signal, Properties};
use gtk::{glib, subclass::prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::models::{Record, Task};
use crate::db::operations::read_tasks;
use crate::views::project::{TaskRow, TaskWindow};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/tasks_list.ui")]
    #[properties(wrapper_type=super::TasksList)]
    pub struct TasksList {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[property(get, set)]
        pub duration: Cell<i64>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub duration_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TasksList {
        const NAME: &'static str = "TasksList";
        type Type = super::TasksList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, _| {
                obj.imp().tasks_box.invalidate_sort();
            });
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

    impl ObjectImpl for TasksList {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_tasks_box_funcs();
            obj.add_bindings();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("task-moveout")
                    .param_types([TaskRow::static_type()])
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
    impl WidgetImpl for TasksList {}
    impl BoxImpl for TasksList {}
}

glib::wrapper! {
    pub struct TasksList(ObjectSubclass<imp::TasksList>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl TasksList {
    pub fn new(datetime: glib::DateTime) -> Self {
        let obj: TasksList = glib::Object::new::<Self>();
        let imp = obj.imp();
        let end = datetime.add_days(1).unwrap().to_unix();

        let now = glib::DateTime::now_local().unwrap();
        if now.ymd() == datetime.ymd() {
            let name_format = format!("%e %b, {}", gettext("Today"));
            imp.name.set_label(&datetime.format(&name_format).unwrap());
        } else {
            imp.name.set_label(&datetime.format("%e %b, %A").unwrap());
        }

        let tasks = read_tasks(None, None, None, None, Some((datetime.to_unix(), end)))
            .expect("Failed to read tasks");
        let mut duration = 0;
        if tasks.is_empty() {
            imp.name.add_css_class("dim-label");
            imp.tasks_box.set_visible(false);
        } else {
            for task in tasks {
                duration += task.duration();
                let task_row = TaskRow::new(task, false);
                imp.tasks_box.append(&task_row);
            }
        }
        obj.set_duration(duration);

        obj.set_datetime(datetime);
        obj
    }

    pub fn add_row(&self, row: TaskRow) {
        let imp = self.imp();
        imp.tasks_box.append(&row);
        imp.tasks_box.set_visible(true);
        imp.name.remove_css_class("dim-label");
        self.set_duration(self.duration() + row.task().duration());
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

    fn set_tasks_box_funcs(&self) {
        let imp = self.imp();
        imp.tasks_box.set_filter_func(|row| {
            let row = row.downcast_ref::<TaskRow>().unwrap();
            if row.task().suspended() {
                false
            } else {
                !row.imp().moving_out.get()
            }
        });
    }

    #[template_callback]
    fn handle_tasks_box_row_activated(&self, row: gtk::ListBoxRow, tasks_box: gtk::ListBox) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let row = row.downcast::<TaskRow>().unwrap();
        let modal = TaskWindow::new(&win.application().unwrap(), &win, row.task());
        modal.present();
        row.cancel_timer();
        let page_datetime = self.datetime().to_unix();
        modal.connect_closure(
            "task-window-close",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row, @weak-allow-none tasks_box => move |_win: TaskWindow, task: Task| {
                let tasks_box = tasks_box.unwrap();
                let row = row.unwrap();
                let task_date = task.date();
                let task_duration = task.duration();
                row.reset(task);
                row.changed();
                if task_date != page_datetime {
                    obj.set_duration(obj.duration() - task_duration);
                    tasks_box.remove(&row);
                    if tasks_box.first_child().is_none() {
                        obj.imp().name.add_css_class("dim-label");
                    }
                    obj.emit_by_name::<()>("task-moveout", &[&row]);
                }
            }),
        );
    }
}
