use adw;
use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::db::models::{Project, Record};
use crate::db::operations::{read_records, read_tasks, update_project};
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_header.ui")]
    pub struct ProjectHeader {
        #[template_child]
        pub name_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub duration_button_content: TemplateChild<adw::ButtonContent>,
        #[template_child]
        pub stat_box: TemplateChild<gtk::ListBox>,
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

    impl ObjectImpl for ProjectHeader {}
    impl WidgetImpl for ProjectHeader {}
    impl BoxImpl for ProjectHeader {}
}

glib::wrapper! {
    pub struct ProjectHeader(ObjectSubclass<imp::ProjectHeader>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl ProjectHeader {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    // open_project - used by handle_project_open and handle_project_update in window
    pub fn open_project(&self, project: &Project) {
        let imp = self.imp();

        imp.name_button.set_label(&project.name());
        imp.name_entry.buffer().set_text(&project.name());

        if let Some(duration) = project.duration() {
            imp.duration_button_content
                .set_label(&Record::duration_display(duration));
        } else {
            imp.duration_button_content.set_label("");
        }

        let lists = imp.stat_box.observe_children();
        for _i in 0..lists.n_items() {
            if let Some(row) = lists.item(0).and_downcast::<gtk::ListBoxRow>() {
                imp.stat_box.remove(&row);
            }
        }

        let now = glib::DateTime::now_local().unwrap();
        let dates = &mut vec![];
        dates.push(now.to_unix());
        let tasks = read_tasks(project.id(), None, None).expect("Faield to read tasks");
        for i in 0..7 {
            let date = glib::DateTime::from_local(
                now.year(),
                now.month(),
                now.day_of_month() - i,
                0,
                0,
                0.0,
            )
            .unwrap();
            let date_unix = date.to_unix();
            let mut duration = 0;
            for task in &tasks {
                let records =
                    read_records(task.id(), false, Some(date_unix), Some(dates[(i) as usize]))
                        .expect("Failed to read records");
                for record in records {
                    duration += record.duration();
                }
            }
            if duration != 0 {
                let stat_item = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let date_label = if i == 0 {
                    String::from("Today")
                } else {
                    date.format("%A").unwrap().to_string()
                };
                let stat_item_date = gtk::Label::builder().label(&date_label).build();
                stat_item.append(&stat_item_date);
                let stat_item_duration = gtk::Label::builder()
                    .label(&Record::duration_display(duration))
                    .hexpand(true)
                    .halign(gtk::Align::End)
                    .build();
                stat_item.append(&stat_item_duration);
                imp.stat_box.append(&stat_item);
            }
            dates.push(date_unix);
        }
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let name = entry.buffer().text();
        let win = self.root().and_downcast::<IPlanWindow>().unwrap();
        let project = win.project();
        let imp = self.imp();
        imp.name_button.set_label(&name);
        imp.name_button.set_visible(true);
        project.set_property("name", name);
        update_project(&project).expect("Failed to update project");
        win.imp()
            .sidebar
            .imp()
            .projects_section
            .update_project(&project);
    }
}
