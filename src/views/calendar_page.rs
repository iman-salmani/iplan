use adw::prelude::*;
use gtk::glib::Properties;
use gtk::{glib, subclass::prelude::*};
use std::cell::RefCell;

use crate::views::{project::TaskRow, TasksList};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar_page.ui")]
    #[properties(wrapper_type=super::CalendarPage)]
    pub struct CalendarPage {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[template_child]
        pub tasks_lists: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarPage {
        const NAME: &'static str = "CalendarPage";
        type Type = super::CalendarPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                tasks_lists: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for CalendarPage {
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
    impl WidgetImpl for CalendarPage {}
    impl BoxImpl for CalendarPage {}
}

glib::wrapper! {
    pub struct CalendarPage(ObjectSubclass<imp::CalendarPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl CalendarPage {
    pub fn new(datetime: glib::DateTime) -> Self {
        let obj: CalendarPage = glib::Object::new::<Self>();
        let imp = obj.imp();
        for i in 0..7 {
            let tasks_list = TasksList::new(datetime.add_days(i).unwrap());
            imp.tasks_lists.append(&tasks_list);
            tasks_list.connect_closure(
                "task-moveout",
                false,
                glib::closure_local!(@watch obj => move |_: TasksList, row: TaskRow| {
                    let start = obj.datetime();
                    let task = row.task();
                    let task_date = task.date();
                    let difference = task_date - start.to_unix();
                    let mut i = difference / (24 * 60 * 60);
                    if i >= 0 && i < 7 {
                        let tasks_list = obj.imp().tasks_lists.observe_children().item(i as u32).and_downcast::<TasksList>().unwrap();
                        tasks_list.add_row(row);
                    } else {
                        i = i / 7;
                        if i <= 0 {
                            i -= 1;
                        }
                        let target_week = start.add_days(i as i32 * 7).unwrap();
                        let stack = obj.parent().and_downcast::<gtk::Stack>().unwrap();
                        let name = target_week.format("%F").unwrap();
                        if let Some(page) = stack.child_by_name(&name) {
                            let page = page.downcast::<Self>().unwrap();
                            let difference = task_date - target_week.to_unix();
                            let i = difference / 86400; // day in seconds
                            let tasks_lists = page.imp().tasks_lists.observe_children();
                            let tasks_list = tasks_lists.item(i.abs() as u32);
                            let tasks_list = tasks_list.and_downcast::<TasksList>().unwrap();
                            tasks_list.add_row(row);
                        }
                    }
                }),
            );
        }
        obj.set_datetime(datetime);
        obj
    }
}
