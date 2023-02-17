use gtk::{glib, gdk, prelude::*, subclass::prelude::*, glib::once_cell::sync::Lazy};
use adw::prelude::*;
use std::cell::{Cell, RefCell};

use crate::db::models::{List, Task};
use crate::db::operations::{update_list, delete_list, create_task, read_tasks, read_task, update_task, new_position};
use crate::views::{IPlanWindow, project::ProjectListTask};

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
                    !row.imp().moving_out.get()
                }
            }
        ));

        self.fetch(project_id, false);

        let task_drop_target =
            gtk::DropTarget::new(ProjectListTask::static_type(), gdk::DragAction::MOVE);
        task_drop_target.set_preload(true);
        task_drop_target.connect_drop(glib::clone!(
            @weak self as obj => @default-return false,
            move |target, value, x, y| obj.task_drop_target_drop(target, value, x, y)));
        task_drop_target.connect_motion(glib::clone!(
            @weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_motion(target, x, y)));
        task_drop_target.connect_enter(glib::clone!(
            @weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_enter(target, x, y)));
        task_drop_target.connect_leave(glib::clone!(
            @weak self as obj =>
            move |target| obj.task_drop_target_leave(target)));
        imp.tasks_box.add_controller(&task_drop_target);
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
        let dialog = gtk::Builder::
            from_resource("/ir/imansalmani/iplan/ui/project/project_list_delete_dialog.ui")
            .object::<adw::MessageDialog>("dialog")
            .unwrap();
        dialog.set_transient_for(self.root().and_downcast::<gtk::Window>().as_ref());
        dialog.set_heading(Some(&format!("Delete {} List?", self.list().name())));
        dialog.connect_response(Some("delete"), glib::clone!(
            @weak self as obj =>
            move |_dialog, response| {
                if response == "delete" {
                    delete_list(obj.list().id()).expect("Failed to delete list");
                    let lists_box = obj.parent().and_downcast::<gtk::Box>().unwrap();
                    let placeholder = obj.root()
                        .and_downcast::<IPlanWindow>()
                        .unwrap()
                        .imp()
                        .project_lists
                        .imp()
                        .placeholder
                        .get();
                    lists_box.remove(&obj);
                    if lists_box.first_child().is_none() {
                        lists_box.append(&placeholder);
                    }}}));
        dialog.present();
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

    fn task_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        // Source_row moved by motion signal so it should drop on itself
        self.imp().tasks_box.drag_unhighlight_row();
        let row: ProjectListTask = value.get().unwrap();
        let task = row.task();
        let task_db = read_task(task.id()).expect("Failed to read task");
        if task_db.position() != task.position() || task_db.list() != task.list() {
            update_task(task).expect("Failed to update task");
        }
        row.grab_focus();
        true
    }

    fn task_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        y: f64,
    ) -> gdk::DragAction {
        let imp = self.imp();
        let source_row: Option<ProjectListTask> = target.value_as();
        let target_row = imp.tasks_box.row_at_y(y as i32);

        if source_row.is_none() || target_row.is_none() {
            return gdk::DragAction::empty();
        }
        let source_row = source_row.unwrap();
        let target_row: ProjectListTask = target_row.and_downcast().unwrap();

        // Move
        let source_task = source_row.task();
        let target_task = target_row.task();
        if source_task.id() != target_task.id() {
            let source_i = source_row.index();
            let target_i = target_row.index();
            let source_p = source_task.position();
            let target_p = target_task.position();
            if source_i - target_i == 1 {
                source_task.set_property("position", source_p + 1);
                target_task.set_property("position", target_p - 1);
            } else if source_i > target_i {
                for i in target_i..source_i {
                    let row: ProjectListTask = imp
                        .tasks_box
                        .row_at_index(i)
                        .and_downcast()
                        .unwrap();
                    row.task().set_property("position", row.task().position() - 1);
                }
                source_task.set_property("position", target_p)
            } else if source_i - target_i == -1 {
                source_task.set_property("position", source_p - 1);
                target_task.set_property("position", target_p + 1);
            } else if source_i < target_i {
                for i in source_i + 1..target_i + 1 {
                    let row: ProjectListTask = imp
                        .tasks_box
                        .row_at_index(i)
                        .and_downcast()
                        .unwrap();
                    row.task().set_property("position", row.task().position() + 1);
                }
                source_task.set_property("position", target_p)
            }

            // Should use invalidate_sort() insteed of changed() for Refresh hightlight shape
            // TODO: Check this in rust
            imp.tasks_box.invalidate_sort();
        }

        // Scroll
        // FIXME: Dont work on vertical layout
        if imp.tasks_box.height() > imp.scrolled_window.height() {
            let adjustment = imp.scrolled_window.vadjustment();
            let step = adjustment.step_increment() / 3.0;
            let v_pos = adjustment.value();
            if y - v_pos > 475.0 {
                adjustment.set_value(v_pos + step);
            } else if y - v_pos < 25.0 {
                adjustment.set_value(v_pos - step);
            }
        }

        gdk::DragAction::MOVE
    }

    fn task_drop_target_enter (
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        _y: f64,
        ) -> gdk::DragAction {
        let row: ProjectListTask = target.value_as().unwrap();
        let tasks_box = self.imp().tasks_box.get();
        row.imp().moving_out.set(false);
        // Avoid from when drag start
        if row.task().list() == self.list().id() && row.imp().moving_out.get() {
            tasks_box.invalidate_filter();
        } else if row.task().list() != self.list().id() {
            let task = row.task();
            let list_id = self.list().id();
            task.set_property("list", list_id);
            task.set_property("position", new_position(list_id));
            let parent = row.parent().and_downcast::<gtk::ListBox>().unwrap();
            for i in 0..row.index() {
                let task = parent.row_at_index(i)
                    .and_downcast::<ProjectListTask>()
                    .unwrap()
                    .task();
                task.set_property("position", task.position() - 1);
            }
            parent.remove(&row);
            tasks_box.prepend(&row);
            tasks_box.drag_highlight_row(&row);
        }
        gdk::DragAction::MOVE
    }

    fn task_drop_target_leave(&self, target: &gtk::DropTarget) {
        if let Some(row) = target.value_as::<ProjectListTask>() {
            row.imp().moving_out.set(true);
            self.imp().tasks_box.invalidate_filter();
        }
    }
}
