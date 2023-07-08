use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
use crate::views::calendar::{CalendarPage, DayIndicator};

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, glib::Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/calendar/calendar.ui")]
    #[properties(wrapper_type=super::Calendar)]
    pub struct Calendar {
        #[property(get, set)]
        pub datetime: RefCell<glib::DateTime>,
        #[template_child]
        pub day_switcher: TemplateChild<gtk::Box>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
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
                |_, _, _| {},
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                datetime: RefCell::new(glib::DateTime::now_local().unwrap()),
                day_switcher: TemplateChild::default(),
                stack: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for Calendar {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.init_widgets();
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

    pub fn open_today(&self) {
        let imp = self.imp();
        let today = self.today_datetime();
        let possible_today_indicator_date = imp
            .day_switcher
            .observe_children()
            .item(2)
            .and_downcast::<DayIndicator>()
            .unwrap()
            .datetime();

        if possible_today_indicator_date != today {
            self.clear_day_switcher();
            for day in -2..5 {
                let datetime = today.add_days(day).unwrap();
                imp.day_switcher.append(&self.new_day_indicator(datetime));
            }
        }

        let datetime = today.add_days(-2).unwrap();
        if self.datetime() != datetime {
            self.set_page(datetime);
            self.refresh_indicators_selection();
        } else if possible_today_indicator_date != today {
            self.refresh_indicators_selection();
        }
    }

    pub fn refresh(&self) {
        let imp = self.imp();
        let datetime = self.datetime();
        let name = datetime.format("%F").unwrap();
        let new_page = CalendarPage::new(datetime);
        let pages = imp.stack.observe_children();
        for _ in 0..pages.n_items() {
            imp.stack.remove(&imp.stack.first_child().unwrap());
        }
        imp.stack.add_named(&new_page, Some(&name));
    }

    fn init_widgets(&self) {
        let imp = self.imp();
        let today = self.today_datetime();
        for day in -2..5 {
            let datetime = today.add_days(day).unwrap();
            imp.day_switcher.append(&self.new_day_indicator(datetime));
        }
    }

    fn new_day_indicator(&self, datetime: glib::DateTime) -> DayIndicator {
        let day_indicator = DayIndicator::new(datetime);
        day_indicator.connect_clicked(glib::clone!(@weak self as obj => move |_indicator| {
            // let datetime = indicator.datetime();
            // obj.set_page(datetime);
            // obj.refresh_indicators_selection();
        }));
        day_indicator
    }

    fn set_page(&self, datetime: glib::DateTime) {
        let imp = self.imp();
        let previous_datetime = self.datetime();

        if previous_datetime == datetime {
            return;
        }

        let name = datetime.format("%F").unwrap();
        let transition: gtk::StackTransitionType = if previous_datetime < datetime {
            gtk::StackTransitionType::SlideUp
        } else {
            gtk::StackTransitionType::SlideDown
        };

        self.set_datetime(&datetime);
        if imp.stack.child_by_name(&name).is_none() {
            let page = CalendarPage::new(datetime);
            imp.stack.add_named(&page, Some(&name));
        }
        imp.stack.set_visible_child_full(&name, transition);
    }

    fn refresh_indicators_selection(&self) {
        let imp = self.imp();
        // let name = imp.stack.visible_child_name().unwrap();
        let indicators = imp.day_switcher.observe_children();
        let today = self.today_datetime();
        for i in 0..indicators.n_items() {
            let indicator = indicators.item(i).and_downcast::<DayIndicator>().unwrap();
            // if indicator.datetime().format("%F").unwrap() == name {
            //     indicator.remove_css_class("flat");
            // } else {
            //     indicator.add_css_class("flat");
            // }
            if today == indicator.datetime() {
                indicator.add_css_class("accent");
            }
        }
    }

    fn switcher_next(&self) {
        let imp = self.imp();
        let last_indicator = imp
            .day_switcher
            .last_child()
            .and_downcast::<DayIndicator>()
            .unwrap();

        self.clear_day_switcher();

        for i in 1..8 {
            let datetime = last_indicator.datetime().add_days(i).unwrap();
            if i == 1 {
                self.set_page(datetime.clone());
            }
            let new_indicator = self.new_day_indicator(datetime);
            imp.day_switcher.append(&new_indicator);
        }
    }

    fn switcher_previous(&self) {
        let imp = self.imp();
        let first_indicator = imp
            .day_switcher
            .first_child()
            .and_downcast::<DayIndicator>()
            .unwrap();

        self.clear_day_switcher();

        for i in 1..8 {
            let datetime = first_indicator.datetime().add_days(-i).unwrap();
            if i == 7 {
                self.set_page(datetime.clone());
            }
            let new_indicator = self.new_day_indicator(datetime);
            imp.day_switcher.prepend(&new_indicator);
        }
    }

    fn clear_day_switcher(&self) {
        let imp = self.imp();
        loop {
            if let Some(indicator) = imp.day_switcher.first_child() {
                imp.day_switcher.remove(&indicator);
            } else {
                break;
            }
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
    fn handle_next_day_clicked(&self, _: gtk::Button) {
        self.switcher_next();
        self.refresh_indicators_selection();
    }

    #[template_callback]
    fn handle_previous_day_clicked(&self, _: gtk::Button) {
        self.switcher_previous();
        self.refresh_indicators_selection();
    }
}
