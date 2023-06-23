use adw::prelude::*;
use gtk::glib::Properties;
use gtk::{gdk, glib, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::Duration;

use crate::views::{calendar::TasksList, task::TaskRow};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar/calendar_page.ui")]
    #[properties(wrapper_type=super::CalendarPage)]
    pub struct CalendarPage {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
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
                scroll: Cell::new(0),
                scrolled_window: TemplateChild::default(),
                tasks_lists: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for CalendarPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_controllers();
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

    fn add_controllers(&self) {
        let controller = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        controller.set_preload(true);
        controller.connect_motion(|controller, _, y| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            let height = obj.height();
            if height - (y as i32) < 50 {
                if obj.scroll() != 1 {
                    obj.set_scroll(1);
                    obj.start_scroll();
                }
            } else if y < 50.0 {
                if obj.scroll() != -1 {
                    obj.set_scroll(-1);
                    obj.start_scroll();
                }
            } else {
                obj.set_scroll(0)
            }
            gdk::DragAction::empty()
        });
        controller.connect_leave(|controller| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            obj.set_scroll(0);
        });
        self.add_controller(controller);
    }

    fn start_scroll(&self) {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || loop {
            if tx.send(()).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.1));
        });
        rx.attach(
            None,
            glib::clone!(@weak self as obj => @default-return glib::Continue(false), move |_| {
                let scroll = obj.scroll();
                if scroll == 0 {
                    glib::Continue(false)
                } else if scroll.is_positive() {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepDown, false);
                    glib::Continue(true)
                } else {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepUp, false);
                    glib::Continue(true)
                }
            }),
        );
    }
}
