use adw::prelude::*;
use gettextrs::gettext;
use gtk::glib::Properties;
use gtk::{glib, subclass::prelude::*};
use std::cell::RefCell;

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
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub duration: TemplateChild<gtk::Label>,
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
                name: TemplateChild::default(),
                duration: TemplateChild::default(),
                tasks_box: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for TasksList {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_tasks_box_funcs();
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
            imp.duration.set_visible(false);
            imp.tasks_box.set_visible(false);
        } else {
            for task in tasks {
                duration += task.duration();
                let project_list_task = TaskRow::new(task, false);
                imp.tasks_box.append(&project_list_task);
            }
            if duration == 0 {
                imp.duration.set_visible(false);
            } else {
                imp.duration.set_label(&Record::duration_display(duration));
            }
        }

        obj.set_datetime(datetime);
        obj
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
            glib::closure_local!(@watch row => move |_win: TaskWindow, task: Task| {
                if task.date() == page_datetime {
                    row.reset(task);
                    row.changed();
                } else {
                    tasks_box.remove(row);
                }
            }),
        );
    }
}
