use gtk::{glib, prelude::*, subclass::prelude::*, glib::once_cell::sync::Lazy};
use std::cell::{Cell, RefCell};

use crate::db::models::{List, Task};
use crate::db::operations::{create_task, read_tasks, update_list};
use crate::views::project::ProjectListTask;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_list.ui")]
    pub struct ProjectList {
        pub list: RefCell<List>,
        pub contain_done_tasks: Cell<bool>,
        #[template_child]
        pub header: TemplateChild<gtk::Box>,
        #[template_child]
        pub name_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub new_task_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub options_button: TemplateChild<gtk::MenuButton>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub options_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub show_done_tasks_toggle_button: TemplateChild<gtk::ToggleButton>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectList {
        const NAME: &'static str = "ProjectList";
        type Type = super::ProjectList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.done", Some("i"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get().unwrap();
                let upper_row = imp.tasks_box.row_at_index(value - 1);
                let row = imp.tasks_box.row_at_index(value).unwrap();
                if obj.contain_done_tasks() {
                    if let Some(upper_row) = upper_row {upper_row.grab_focus();}
                    row.changed();
                } else if !imp.show_done_tasks_toggle_button.is_active() {
                    if let Some(upper_row) = upper_row {upper_row.grab_focus();}
                    imp.tasks_box.remove(&row);
                }
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectList {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![
                    glib::ParamSpecObject::builder::<List>("list").build(),
                    glib::ParamSpecBoolean::builder("contain-done-tasks").build(),
                ]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "list" => {
                    let value = value.get::<List>().expect("value must be a List");
                    self.list.replace(value);
                }
                "contain-done-tasks" => {
                    let value = value.get::<bool>().expect("value must be a bool");
                    self.contain_done_tasks.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "list" => self.list.borrow().to_value(),
                "contain-done-tasks" => self.contain_done_tasks.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ProjectList {}
    impl BoxImpl for ProjectList {}
}

glib::wrapper! {
    pub struct ProjectList(ObjectSubclass<imp::ProjectList>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl ProjectList {
    pub fn new(list: List) -> Self {
        glib::Object::builder()
            .property("list", list)
            .build()
    }

    pub fn init_widgets(&self, project_id: i64) {
        let imp = self.imp();
        let list = self.list();

        imp.name_button.set_label(&list.name());
        imp.name_entry.buffer().set_text(&list.name());

        imp.tasks_box.set_sort_func(|row1, row2| {
            let row1_p = row1.property::<Task>("task").position();
            let row2_p = row2.property::<Task>("task").position();

            if row1_p < row2_p {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });

        imp.tasks_box.set_filter_func(glib::clone!(
            @weak imp => @default-return false,
            move |row| {
                let row = row.downcast_ref::<ProjectListTask>().unwrap();
                if row.task().suspended() {
                    false
                } else if !imp.show_done_tasks_toggle_button.is_active() {
                    !row.task().done()
                } else {
                    true
                    // TODO: replace with !row.moving_out()
                }
            }
        ));

        self.fetch(project_id, false);
    }

    pub fn list(&self) -> List {
        self.property("list")
    }

    pub fn contain_done_tasks(&self) -> bool {
        self.property("contain-done-tasks")
    }

    // TODO: focus_on_task

    // TODO: set_scroll_controller

    fn fetch(&self, project_id: i64, done_tasks: bool) {
        let imp = self.imp();
        for task in read_tasks(project_id, None, Some(done_tasks)).expect("Faield to read tasks") {
            let project_list_task = ProjectListTask::new(task);
            imp.tasks_box.append(&project_list_task);
            project_list_task.init_widgets();
        }
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false);  // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let name = entry.buffer().text();
        let list = self.list();
        let imp = self.imp();
        imp.name_button.set_label(&name);
        imp.name_button.set_visible(true);
        list.set_property("name", name);
        update_list(list).expect("Failed to update list");
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        let list = self.list();
        let task = create_task("", list.project(), list.id())
            .expect("Failed to create task");
        let task_ui = ProjectListTask::new(task);
        let imp = self.imp();
        imp.tasks_box.prepend(&task_ui);
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        imp.options_button.popdown();
        // TODO: present ProjectListDeleteDialog
    }

    #[template_callback]
    fn handle_show_done_tasks_toggle_button_toggled(&self, _button: gtk::ToggleButton) {
        let imp = self.imp();
        imp.options_button.popdown();
        if !self.contain_done_tasks() {
            self.set_property("contain_done_tasks", true);
            self.fetch(self.list().project(), true);
        } else {
            imp.tasks_box.invalidate_filter();
        }
    }

    // TODO: handle_scroll_controller_scroll

    // TODO: handle_drag_list_source_prepare

    // TODO: handle_drag_list_source_begin

    // TODO: handle_drop_list_target_drop

    // TODO: handle_drop_list_target_motion

    // TODO: handle_drop_task_target_drop

    // TODO: handle_drop_task_target_motion

    // TODO: handle_drop_task_target_enter

    // TODO: handle_drop_task_target_leave
}

