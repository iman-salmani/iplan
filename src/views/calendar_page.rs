use adw::prelude::*;
use gtk::glib::Properties;
use gtk::{glib, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
use crate::db::operations::{read_task, read_tasks};
use crate::views::project::{TaskRow, TaskWindow};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar_page.ui")]
    #[properties(wrapper_type=super::CalendarPage)]
    pub struct CalendarPage {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CalendarPage {
        const NAME: &'static str = "CalendarPage";
        type Type = super::CalendarPage;
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
                tasks_box: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for CalendarPage {
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
        let end = datetime.add_days(1).unwrap().to_unix();
        let tasks = read_tasks(None, None, None, None, Some((datetime.to_unix(), end)))
            .expect("Failed to read tasks");
        for task in tasks {
            let project_list_task = TaskRow::new(task, false);
            imp.tasks_box.append(&project_list_task);
        }
        obj.set_datetime(datetime);
        obj
    }

    fn set_tasks_box_funcs(&self) {
        let imp = self.imp();
        imp.tasks_box.set_sort_func(|row1, _| {
            let row1_done = row1.property::<Task>("task").done();

            if row1_done {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });

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
        modal.connect_close_request(glib::clone!(
            @weak row as obj => @default-return gtk::Inhibit(false),
            move |_| {
                let task = read_task(obj.task().id()).expect("Failed to read the task");
                if task.date() == page_datetime {
                    obj.reset(task);
                    obj.changed();
                } else {
                    tasks_box.remove(&obj);
                }
                gtk::Inhibit(false)
            }
        ));
    }
}
