use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::Cell;
use std::thread;

use crate::db::models::{Project, Record};
use crate::db::operations::{read_records, read_tasks};
use crate::views::snippets::Chart;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_header.ui")]
    #[properties(type_wrapper=super::ProjectHeader)]
    pub struct ProjectHeader {
        #[property(get, set)]
        pub project_id: Cell<i64>,
        #[property(get, set)]
        pub stat_updated: Cell<bool>,
        #[template_child]
        pub name_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub icon_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub project_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub description: TemplateChild<gtk::Label>,
        #[template_child]
        pub total_time: TemplateChild<gtk::Label>,
        #[template_child]
        pub chart_header: TemplateChild<gtk::Box>,
        #[template_child]
        pub chart_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub chart_subtitle: TemplateChild<gtk::Label>,
        #[template_child]
        pub chart: TemplateChild<Chart>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectHeader {
        const NAME: &'static str = "ProjectHeader";
        type Type = super::ProjectHeader;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectHeader {
        fn constructed(&self) {
            self.parent_constructed();
            self.project_popover.set_offset(0, 3);
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
    impl WidgetImpl for ProjectHeader {}
    impl BoxImpl for ProjectHeader {}
}

glib::wrapper! {
    pub struct ProjectHeader(ObjectSubclass<imp::ProjectHeader>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

impl Default for ProjectHeader {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

#[gtk::template_callbacks]
impl ProjectHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_project(&self, project: &Project) {
        let imp = self.imp();

        imp.icon_label.set_label(&project.icon());
        imp.name_label.set_label(&project.name());
        let project_description = project.description();
        if project_description.is_empty() {
            imp.description.set_visible(false);
        } else {
            imp.description.set_label(&project.description());
            imp.description.set_visible(true);
        }
        self.set_project_id(project.id());
    }

    #[template_callback]
    fn handle_popover_show(&self, _popover: gtk::Popover) {
        if self.stat_updated() {
            return;
        } else {
            self.set_stat_updated(true);
        }

        let imp = self.imp();
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let project_id = self.project_id();

        imp.chart.clear();
        imp.chart_header.set_visible(false);

        thread::spawn(move || {
            let now = glib::DateTime::now_local().unwrap();
            let dates = &mut vec![];
            dates.push(now.to_unix());
            let tasks: Vec<crate::db::models::Task> =
                read_tasks(Some(project_id), None, None, None, None, false).unwrap();

            let mut last_7_days = 0;
            let mut labels = vec![];
            let mut values = vec![];
            let mut tooltips = vec![];
            for i in 0..7 {
                let date = glib::DateTime::from_local(
                    now.year(),
                    now.month(),
                    now.day_of_month(),
                    0,
                    0,
                    0.0,
                )
                .unwrap();
                let date = date.add_days(-i).unwrap();
                let date_unix = date.to_unix();
                let mut duration = 0;
                for task in &tasks {
                    let records = read_records(
                        Some(task.id()),
                        false,
                        Some(date_unix),
                        Some(dates[(i) as usize]),
                    )
                    .expect("Failed to read records");
                    for record in records {
                        duration += record.duration();
                    }
                }
                last_7_days += duration;
                dates.push(date_unix);
                labels.push(date.format("%e ").unwrap().to_string());
                values.push(duration);
                tooltips.push(Record::duration_display(duration));
            }

            let mut total_time = 0;
            for task in tasks {
                total_time += task.duration();
            }
            tx.send((total_time, last_7_days, labels, values, tooltips))
                .unwrap();
        });
        rx.attach(
            None,
            glib::clone!(
            @weak imp => @default-return glib::Continue(false),
            move |data| {
                let (total_time, last_7_days, labels, values, tooltips) = data;

                if total_time == 0 {
                    imp.placeholder.set_visible(true);
                    return glib::Continue(false);
                } else {
                    imp.placeholder.set_visible(false);
                    imp.chart_header.set_visible(true);
                }

                imp.total_time.set_label(&Record::duration_display(total_time));
                imp.chart_title.set_label(&Record::duration_display(last_7_days));

                let now = glib::DateTime::now_local().unwrap();
                let start = now.add_days(-6).unwrap();
                imp.chart_subtitle.set_label(&format!("{} - {}", start.format("%d").unwrap(), now.format("%d %b %Y").unwrap()));    // FIXME: add start month name if month changed or even year!

                let bigger_duration = values.iter().max().unwrap().to_owned(); // FIXME: Check why returns option
                let max_hour = (bigger_duration / 3600) + 2;
                let max = max_hour * 3600;
                let mut y_labels = vec![];
                for i in 0..max_hour {
                    y_labels.push(i.to_string());
                }
                let percentages = values.iter().map(|value| {
                    let value = value.to_owned() as f64;
                    value / max as f64
                }).collect();
                imp.chart.set_data(labels, percentages, y_labels, tooltips);
                glib::Continue(false)
            }),
        );
    }
}
