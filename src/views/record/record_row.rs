use adw;
use adw::subclass::prelude::*;
use adw::traits::{ActionRowExt, PreferencesRowExt};
use glib::{once_cell::sync::Lazy, subclass::Signal};

use gettextrs::gettext;
use gtk::{glib, glib::Properties, prelude::*};
use std::cell::RefCell;

use crate::db::models::{Record, Task};
use crate::db::operations::delete_record;
use crate::views::record::RecordWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/record/record_row.ui")]
    #[properties(wrapper_type=super::RecordRow)]
    pub struct RecordRow {
        #[property(get, set)]
        pub record: RefCell<Record>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RecordRow {
        const NAME: &'static str = "RecordRow";
        type Type = super::RecordRow;
        type ParentType = adw::ActionRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RecordRow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![
                    Signal::builder("changed").build(),
                    Signal::builder("deleted").build(),
                ]
            });
            SIGNALS.as_ref()
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
    impl WidgetImpl for RecordRow {}
    impl ListBoxRowImpl for RecordRow {}
    impl PreferencesRowImpl for RecordRow {}
    impl ActionRowImpl for RecordRow {}
}

glib::wrapper! {
    pub struct RecordRow(ObjectSubclass<imp::RecordRow>)
        @extends gtk::Widget, gtk::ListBoxRow, adw::PreferencesRow, adw::ActionRow,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

#[gtk::template_callbacks]
impl RecordRow {
    pub fn new(record: Record) -> Self {
        let obj: Self = glib::Object::builder().property("record", record).build();
        obj.set_labels();
        obj
    }

    fn set_labels(&self) {
        let record = self.record();
        let start = glib::DateTime::from_unix_local(record.start()).unwrap();
        let duration = record.duration();

        self.set_title(&Record::duration_display(duration));

        let start_date_text = Task::date_display(&start);
        let end = start.add_seconds(duration as f64).unwrap();
        println!("{:?} - {:?}", start.ymd(), end.ymd());
        let end_date_text = if start.ymd() == end.ymd() {
            String::new()
        } else {
            println!("create end date text: {}", Task::date_display(&end));
            format!("{} ", Task::date_display(&end))
        };
        self.set_subtitle(&format!(
            "{} {} {} {}{}",
            start_date_text,
            start.format("%H:%M").unwrap(),
            gettext("until"),
            end_date_text,
            end.format("%H:%M").unwrap()
        ));
    }

    #[template_callback]
    fn handle_activated(&self) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = RecordWindow::new(&win.application().unwrap(), &win, self.record(), true);
        modal.present();
        modal.connect_closure(
            "record-updated",
            true,
            glib::closure_local!(@watch self as obj => move |_win: RecordWindow, record: Record| {
                obj.set_record(record);
                obj.set_labels();
                obj.changed();
                obj.emit_by_name::<()>("changed", &[]);
            }),
        );
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _: gtk::Button) {
        let record = self.record();
        delete_record(record.id()).unwrap();
        self.emit_by_name::<()>("deleted", &[]);
    }
}
