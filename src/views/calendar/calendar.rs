use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::thread;
use std::time::Duration;

use crate::db::models::Task;
use crate::db::operations::{read_records, read_task, task_tree};
use crate::views::calendar::{DayIndicator, DayView};
use crate::views::task::TaskRow;
use crate::views::ActionScope;

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
        pub page_header: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub toggle_sidebar_button: TemplateChild<gtk::ToggleButton>,
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
                Some(Task::static_variant_type().as_str()),
                |obj, _, value| {
                    let task: Task = value.unwrap().get().unwrap();
                    obj.parent()
                        .unwrap()
                        .activate_action(
                            "task.changed",
                            Some(&glib::Variant::from((
                                value.unwrap(),
                                ActionScope::Calendar.to_variant(),
                            ))),
                        )
                        .unwrap();
                    obj.reset_task(task);
                },
            );
            klass.install_action(
                "task.duration-changed",
                Some(Task::static_variant_type().as_str()),
                |obj, _, value| {
                    let task: Task = value.unwrap().get().unwrap();
                    obj.parent()
                        .unwrap()
                        .activate_action(
                            "task.duration-changed",
                            Some(&glib::Variant::from((
                                value.unwrap(),
                                ActionScope::Calendar.to_variant(),
                            ))),
                        )
                        .unwrap();
                    obj.refresh_days_views_duration(task.id());
                    obj.refresh_parents_timers(task.parent());
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
                page_header: TemplateChild::default(),
                toggle_sidebar_button: TemplateChild::default(),
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

impl Default for Calendar {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl Calendar {
    pub fn new() -> Self {
        Self::default()
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
            if tx.send(()).is_ok() {
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

    pub fn reset_task(&self, task: Task) {
        let reset_parent_subtasks = |parent_id: i64| {
            if parent_id == 0 {
                return;
            }

            if let Some((_, parent_row)) = self.task_row(parent_id) {
                parent_row.reset_subtasks();
            }
        };

        let target_day_view = |task_date: i64| {
            if task_date != 0 {
                self.day_view_by_date(glib::DateTime::from_unix_local(task_date).unwrap())
            } else {
                None
            }
        };

        let task_id = task.id();
        if task.suspended() {
            self.set_subtasks_suspended(task_id, true);
        }

        if let Some((day_view, row)) = self.task_row(task_id) {
            let old_task = row.task();
            let difference = task.different_properties(&old_task);

            reset_parent_subtasks(task.parent());

            if difference.is_empty() {
                return;
            }

            if difference.contains(&"date") {
                let task_date = task.date();
                day_view.remove_row(&row);

                if let Some(day_view) = target_day_view(task_date) {
                    row.reset(task);
                    day_view.add_row(&row);
                }
                return;
            }

            row.reset(task);

            if difference.contains(&"suspended") {
                row.changed();
            }
        } else {
            reset_parent_subtasks(task.parent());
            let task_date = task.date();

            if task_date == 0 {
                return;
            }

            if let Some(day_view) = target_day_view(task.date()) {
                let row = TaskRow::new(task, false, true);
                day_view.add_row(&row);
            }
        }
    }

    pub fn set_subtasks_suspended(&self, task_id: i64, suspended: bool) {
        let subtasks = task_tree(task_id, true).unwrap();
        for subtask in subtasks {
            if let Some((_, row)) = self.task_row(subtask) {
                row.task().set_suspended(suspended);
                row.changed();
            }
        }
    }

    pub fn refresh_task_timer(&self, task_id: i64) {
        if let Some((_, row)) = self.task_row(task_id) {
            row.refresh_timer();
        }
    }

    pub fn refresh_parents_timers(&self, mut parent_id: i64) {
        while parent_id != 0 {
            parent_id = if let Some((_, row)) = self.task_row(parent_id) {
                row.refresh_timer();
                row.task().parent()
            } else {
                read_task(parent_id).unwrap().parent()
            };
        }
    }

    pub fn refresh_days_views_duration(&self, task_id: i64) {
        let records = read_records(Some(task_id), false, None, None).unwrap(); // FIXME: find an efficient way. like record.changed
        for record in records {
            let start = glib::DateTime::from_unix_local(record.start()).unwrap();
            let start_date = glib::DateTime::new(
                &glib::TimeZone::local(),
                start.year(),
                start.month(),
                start.day_of_month(),
                0,
                0,
                0.0,
            )
            .unwrap();
            if let Some(day_view) = self.day_view_by_date(start_date) {
                day_view.refresh_duration();
            }
        }
    }

    pub fn task_row(&self, task_id: i64) -> Option<(DayView, TaskRow)> {
        let imp = self.imp();
        let days_views = imp.days_box.observe_children();
        for i in 0..days_views.n_items() {
            let day_view = days_views.item(i).and_downcast::<DayView>().unwrap();
            if let Some(row) = day_view.task_row(task_id) {
                return Some((day_view, row));
            }
        }
        None
    }

    fn init_widgets(&self) {
        let imp = self.imp();
        let today = self.today_datetime();
        for day in -2..5 {
            let datetime = today.add_days(day).unwrap();
            imp.navigation_bar.append(&self.new_day_indicator(datetime));
        }
        imp.scrolled_view.vscrollbar().set_sensitive(false);
        self.refresh();
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

                match difference.cmp(&0) {
                    Ordering::Greater => {
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
                    },
                    Ordering::Less => {
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
                    },
                    Ordering::Equal => {}
                }

                let first_day_view = imp.days_box.first_child().and_downcast::<DayView>().unwrap();
                let first_day_view_date = first_day_view.datetime();
                if top_edge_day_view_date.difference(&first_day_view_date).as_days() < 7 {
                    let date = first_day_view_date.add_days(-1).unwrap();
                    let day_view = DayView::new(date);
                    imp.days_box.prepend(&day_view);
                    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
                    glib::idle_add(move || {
                        if tx.send(()).is_ok() {
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

    #[template_callback]
    fn handle_calendar_today_clicked(&self, _: gtk::Button) {
        let today = self.today_datetime();
        self.foucs_on_date(today);
    }
}
