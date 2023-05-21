use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::Cell;

use crate::db::models::Record;
use crate::db::operations::{read_record, read_records};
use crate::views::{project::RecordCreateWindow, project::RecordRow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/records_window.ui")]
    pub struct RecordsWindow {
        pub task_id: Cell<i64>,
        #[template_child]
        pub records_box: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecordsWindow {
        const NAME: &'static str = "RecordsWindow";
        type Type = super::RecordsWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("record.created", Some("x"), move |win, _, value| {
                let record_id = value.unwrap().get::<i64>().unwrap();
                let imp = win.imp();
                let record = read_record(record_id).expect("Failed to read record");
                let row = RecordRow::new(record);
                imp.records_box.append(&row);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecordsWindow {}
    impl WidgetImpl for RecordsWindow {}
    impl WindowImpl for RecordsWindow {}
}

glib::wrapper! {
    pub struct RecordsWindow(ObjectSubclass<imp::RecordsWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl RecordsWindow {
    pub fn new(application: &gtk::Application, app_window: &gtk::Window, task_id: i64) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.task_id.replace(task_id);
        imp.records_box
            .set_sort_func(|row1: &gtk::ListBoxRow, row2| {
                let row1_start = row1.property::<Record>("record").start();
                let row2_start = row2.property::<Record>("record").start();

                if row1_start > row2_start {
                    gtk::Ordering::Smaller
                } else {
                    gtk::Ordering::Larger
                }
            });
        let records = read_records(task_id, false, None, None).expect("Failed to read records");
        for record in records {
            let row = RecordRow::new(record);
            imp.records_box.append(&row);
        }
        win
    }

    #[template_callback]
    fn handle_add_record_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        let modal = RecordCreateWindow::new(&self.application().unwrap(), self, imp.task_id.get());
        modal.present();
    }
}
