use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::Duration;

use crate::db::models::Task;
use crate::views::calendar::{DayIndicator, DayView};
use crate::views::task::TaskRow;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar/calendar.ui")]
    #[properties(wrapper_type=super::Calendar)]
    pub struct Calendar {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
        #[template_child]
        pub navigation_bar: TemplateChild<gtk::Box>,
        #[template_child]
        pub scrolled_view: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub days_box: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Calendar {
        const NAME: &'static str = "Calendar";
        type Type = super::Calendar;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action(
                "task.changed",
                Some(&Task::static_variant_type_string()),
                |obj, _, value| {
                    let task = Task::try_from(value.unwrap()).unwrap();

                    let reset_parent_subtasks = |parent_id: i64| {
                        if parent_id == 0 {
                            return;
                        }

                        if let Some(parent_row) = obj.task_row(parent_id) {
                            parent_row.reset_subtasks();
                        }
                    };

                    let target_day_view = |task_date: i64| {
                        if task_date != 0 {
                            obj.day_view_by_date(glib::DateTime::from_unix_local(task_date).unwrap())
                        } else {
                            None
                        }
                    };

                    if let Some(row) = obj.task_row(task.id()) {
                        let old_task = row.task();
                        let difference = task.different_properties(&old_task);
                        
                        reset_parent_subtasks(task.parent());

                        if difference.is_empty() {
                            return;
                        }
                        
                        if difference.contains(&"date"){
                            let rows_box = row.parent().and_downcast::<gtk::ListBox>().unwrap();
                            rows_box.remove(&row);

                            if let Some(day_view) = target_day_view(task.date()) {
                                row.reset(task);
                                day_view.add_row(row);
                            }

                        } else {
                            row.reset(task);
                        }

                        return;
                    }

                    reset_parent_subtasks(task.parent());

                    if let Some(day_view) = target_day_view(task.date()) {
                        let row = TaskRow::new(task, false, true);
                        day_view.add_row(row);
                    }
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                scroll: Cell::new(0),
                navigation_bar: TemplateChild::default(),
                scrolled_view: TemplateChild::default(),
                days_box: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for Calendar {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.init_widgets();
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
    impl WidgetImpl for Calendar {}
    impl BoxImpl for Calendar {}
}

glib::wrapper! {
    pub struct Calendar(ObjectSubclass<imp::Calendar>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl Calendar {
    pub fn new() -> Self {
        glib::Object::new::<Self>()
    }

    pub fn go_today(&self) {
        let imp = self.imp();
        let today = self.today_datetime();

        loop {
            if let Some(indicator) = imp.navigation_bar.first_child() {
                imp.navigation_bar.remove(&indicator);
            } else {
                break;
            }
        }

        for day in -2..5 {
            let datetime = today.add_days(day).unwrap();
            imp.navigation_bar.append(&self.new_day_indicator(datetime));
        }
        self.foucs_on_date(today);
    }

    pub fn refresh(&self) {
        let imp = self.imp();
        let datetime = imp
            .navigation_bar
            .first_child()
            .and_downcast::<DayIndicator>()
            .unwrap()
            .datetime();

        let pages = imp.days_box.observe_children();
        for _ in 0..pages.n_items() {
            imp.days_box.remove(&imp.days_box.first_child().unwrap());
        }

        for i in -7..14 {
            let day_view = DayView::new(datetime.add_days(i).unwrap());
            imp.days_box.append(&day_view);
        }

        let day_view = self.day_view_by_date(datetime).unwrap();
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        glib::idle_add(move || {
            if let Ok(_) = tx.send(()) {
                glib::Continue(true)
            } else {
                glib::Continue(false)
            }
        });
        rx.attach(
            None,
            glib::clone!(@weak self as obj, @weak day_view => @default-return glib::Continue(false), move |_| {
                let y =  day_view.allocation().y();
                if y == 0 {
                    glib::Continue(true)
                } else {
                    obj.imp().scrolled_view.vadjustment().set_value(y as f64);
                    glib::Continue(false)
                }
            }),
        );
    }

    fn init_widgets(&self) {
        let imp = self.imp();
        let today = self.today_datetime();
        for day in -2..5 {
            let datetime = today.add_days(day).unwrap();
            imp.navigation_bar.append(&self.new_day_indicator(datetime));
        }
        imp.scrolled_view.vscrollbar().set_sensitive(false);
    }

    fn add_controllers(&self) {
        let imp = self.imp();

        let dnd_controller = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        dnd_controller.set_preload(true);
        dnd_controller.connect_motion(|controller, _, y| {
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
        dnd_controller.connect_leave(|controller| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            obj.set_scroll(0);
        });
        self.add_controller(dnd_controller);

        let vadjustment = imp.scrolled_view.vadjustment();
        vadjustment.connect_value_changed(glib::clone!(@weak self as obj => move |adjustment| {
            let imp = obj.imp();
            let pos = adjustment.value();

            if let Some(top_edge_day_view) = obj.day_view_by_y(pos) {
                let first_day_indicator = imp
                    .navigation_bar
                    .first_child()
                    .and_downcast::<DayIndicator>()
                    .unwrap();
                let top_edge_day_view_date = top_edge_day_view.datetime();
                let difference = top_edge_day_view_date
                    .difference(&first_day_indicator.datetime())
                    .as_days();

                if difference > 0 {
                    for _ in 0..difference {
                        let first_day_indicator = imp
                            .navigation_bar
                            .first_child()
                            .and_downcast::<DayIndicator>()
                            .unwrap();
                        let last_day_indicator = imp
                            .navigation_bar
                            .last_child()
                            .and_downcast::<DayIndicator>()
                            .unwrap();
                        imp.navigation_bar.remove(&first_day_indicator);
    
                        
                        let date = last_day_indicator.datetime().add_days(1).unwrap();
                        let day_indicator = obj.new_day_indicator(date);
                        imp.navigation_bar.append(&day_indicator);
                    }
                } else if difference < 0 {
                    for _ in 0..difference.abs() {
                        let first_day_indicator = imp
                            .navigation_bar
                            .first_child()
                            .and_downcast::<DayIndicator>()
                            .unwrap();
                        let last_day_indicator = imp
                            .navigation_bar
                            .last_child()
                            .and_downcast::<DayIndicator>()
                            .unwrap();
                        imp.navigation_bar.remove(&last_day_indicator);
    
                        let date = first_day_indicator.datetime().add_days(-1).unwrap();
                        let day_indicator = obj.new_day_indicator(date);
                        imp.navigation_bar.prepend(&day_indicator);
                    }
                }

                let first_day_view = imp.days_box.first_child().and_downcast::<DayView>().unwrap();
                let first_day_view_date = first_day_view.datetime();
                if top_edge_day_view_date.difference(&first_day_view_date).as_days() < 7 {
                    let date = first_day_view_date.add_days(-1).unwrap();
                    let day_view = DayView::new(date);
                    imp.days_box.prepend(&day_view);
                    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                    glib::idle_add(move || {
                        if let Ok(_) = tx.send(()) {
                            glib::Continue(true)
                        } else {
                            glib::Continue(false)
                        }
                    });
                    rx.attach(None, glib::clone!(@weak adjustment => @default-return glib::Continue(false), move |_| {
                        let height = day_view.height();
                        if height == 0 {
                            glib::Continue(true)
                        } else {
                            adjustment.set_value(adjustment.value() + height as f64);
                            glib::Continue(false)
                        }
                    }));
                    return;
                }

                let last_day_view = imp.days_box.last_child().and_downcast::<DayView>().unwrap();
                let last_day_view_date = last_day_view.datetime();
                if last_day_view_date.difference(&top_edge_day_view_date).as_days() < 14 {
                    let date = last_day_view_date.add_days(1).unwrap();
                    let day_view = DayView::new(date);
                    imp.days_box.append(&day_view);
                }
            }
        }));
    }

    fn task_row(&self, task_id: i64) -> Option<TaskRow> {
        let imp = self.imp();
        let days_views = imp.days_box.observe_children();
        for i in 0..days_views.n_items() {
            let day_view = days_views.item(i).and_downcast::<DayView>().unwrap();
            if let Some(row) = day_view.imp().tasks_box.item_by_id(task_id) {
                return Some(row);
            }
        }
        None
    }

    fn day_view_by_date(&self, date: glib::DateTime) -> Option<DayView> {
        let imp = self.imp();
        let days_views = imp.days_box.observe_children();
        for i in 0..days_views.n_items() {
            let day_view = days_views.item(i).and_downcast::<DayView>().unwrap();
            if day_view.datetime() == date {
                return Some(day_view);
            }
        }
        None
    }

    fn day_view_by_y(&self, y: f64) -> Option<DayView> {
        let imp = self.imp();

        if let Some(child) = imp.days_box.pick(128.0, y, gtk::PickFlags::DEFAULT) {
            if child.widget_name() == "Viewport" {
                return None;
            }

            let mut widget = child;
            loop {
                let parent = widget.parent();
                if let Ok(day_tasks) = widget.downcast::<DayView>() {
                    return Some(day_tasks);
                }
                if let Some(new_widget) = parent {
                    widget = new_widget;
                } else {
                    break;
                }
            }
        }

        None
    }

    fn new_day_indicator(&self, datetime: glib::DateTime) -> DayIndicator {
        let today = self.today_datetime() == datetime;
        let day_indicator = DayIndicator::new(datetime);
        day_indicator.connect_clicked(glib::clone!(@weak self as obj => move |indicator| {
            obj.foucs_on_date(indicator.datetime());
        }));
        if today {
            day_indicator.add_css_class("accent");
        }
        day_indicator
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
                    obj.imp().scrolled_view.emit_scroll_child(gtk::ScrollType::StepDown, false);
                    glib::Continue(true)
                } else {
                    obj.imp().scrolled_view.emit_scroll_child(gtk::ScrollType::StepUp, false);
                    glib::Continue(true)
                }
            }),
        );
    }

    fn foucs_on_date(&self, date: glib::DateTime) {
        let imp = self.imp();
        self.set_focus_child(Some(&imp.scrolled_view.get()));
        if let Some(day_tasks) = self.day_view_by_date(date) {
            day_tasks.grab_focus();
        }
    }

    fn today_datetime(&self) -> glib::DateTime {
        let now = glib::DateTime::now_local().unwrap();
        glib::DateTime::new(
            &glib::TimeZone::local(),
            now.year(),
            now.month(),
            now.day_of_month(),
            0,
            0,
            0.0,
        )
        .unwrap()
    }
}
