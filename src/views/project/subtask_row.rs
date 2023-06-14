use gtk::{glib, glib::Properties, prelude::*, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::Task;
mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/subtask_row.ui")]
    pub struct SubtaskRow {
        #[template_child]
        pub done: TemplateChild<gtk::Image>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubtaskRow {
        const NAME: &'static str = "SubtaskRow";
        type Type = super::SubtaskRow;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SubtaskRow {}
    impl WidgetImpl for SubtaskRow {}
    impl BoxImpl for SubtaskRow {}
}

glib::wrapper! {
    pub struct SubtaskRow(ObjectSubclass<imp::SubtaskRow>)
        @extends glib::InitiallyUnowned, gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

#[gtk::template_callbacks]
impl SubtaskRow {
    pub fn new(task: Task) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();
        imp.name.set_label(&task.name());
        if task.done() {
            imp.done
                .set_icon_name(Some("check-round-outline-whole-symbolic"));
        } else {
            imp.done
                .set_icon_name(Some("circle-outline-thick-symbolic"));
        }
        obj
    }
}
