use adw::prelude::*;
use gettextrs::gettext;
use gtk::{gdk, glib, glib::Properties, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::{List, Task};
use crate::db::operations::{
    create_task, delete_list, read_list, read_tasks, update_list, update_task,
};
use crate::views::project::{
    ProjectDoneTasksWindow, ProjectLayout, TaskRow, TaskWindow, TasksBox, TasksBoxWrapper,
};
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_list.ui")]
    #[properties(wrapper_type=super::ProjectList)]
    pub struct ProjectList {
        #[property(get, set)]
        pub list: RefCell<List>,
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
        pub tasks_box: TemplateChild<TasksBox>,
        #[template_child]
        pub options_popover: TemplateChild<gtk::Popover>,
        #[template_child]
        pub show_done_tasks_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectList {
        const NAME: &'static str = "ProjectList";
        type Type = super::ProjectList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get().unwrap();
                let upper_row = imp.tasks_box.item_by_index(value - 1);
                let row = imp.tasks_box.item_by_index(value).unwrap();
                let task = row.task();
                if let Some(upper_row) = upper_row {
                    upper_row.grab_focus();
                }
                imp.tasks_box.remove_item(&row);

                let mut toast_name = task.name();
                if toast_name.chars().count() > 15 {
                    toast_name.truncate(15);
                    toast_name.push_str("...");
                }
                let toast = adw::Toast::builder()
                    .title(
                        gettext("\"{}\" moved to the done tasks list").replace("{}", &toast_name),
                    )
                    .button_label(gettext("Undo"))
                    .build();
                toast.connect_button_clicked(glib::clone!(@weak obj, @weak task, @strong row =>
                    move |_toast| {
                        task.set_done(false);
                        update_task(&task).expect("Failed to update task");
                        obj.imp().tasks_box.add_item(&row);
                }));
                let window = obj.root().and_downcast::<IPlanWindow>().unwrap();
                window.imp().toast_overlay.add_toast(toast);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectList {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.add_drag_drop_controllers();
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
    pub fn new(list: List, layout: ProjectLayout, max_height: usize) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_list(list);
        let imp = obj.imp();
        let list = obj.list();

        imp.name_entry.buffer().set_text(&list.name());

        let tasks = read_tasks(
            Some(list.project()),
            Some(list.id()),
            Some(false),
            Some(0),
            None,
        )
        .expect("Failed to read tasks");

        if layout == ProjectLayout::Horizontal {
            imp.tasks_box.send_hscroll();
        } else {
            imp.tasks_box.set_scrollable(false);
        }
        imp.tasks_box
            .set_items_wrapper(TasksBoxWrapper::List(list.id(), list.project()));
        imp.tasks_box.add_tasks_lazy(tasks, max_height);

        obj
    }

    pub fn select_task(&self, target_task: Task) {
        let imp = self.imp();
        let task_rows = imp.tasks_box.observe_children();
        for i in 0..task_rows.n_items() - 1 {
            if let Some(task_row) = task_rows.item(i).and_downcast::<TaskRow>() {
                let list_task = task_row.task();
                if list_task.position() == target_task.position() {
                    task_row.grab_focus();
                    break;
                }
            }
        }
    }

    fn add_drag_drop_controllers(&self) {
        let imp = self.imp();
        let list_drag_source = gtk::DragSource::builder()
            .actions(gdk::DragAction::MOVE)
            .build();
        list_drag_source.connect_prepare(glib::clone!(@weak self as obj => @default-return None,
        move |_drag_source, _x, _y| {
            if obj.imp().name_entry.get_visible() {
                None
            } else {
                Some(gdk::ContentProvider::for_value(&obj.to_value()))
            }
        }));
        list_drag_source.connect_drag_begin(|_drag_source, drag| {
            let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(drag).downcast().unwrap();
            let label = gtk::Label::builder().label("").build();
            drag_icon.set_child(Some(&label));
            drag.set_hotspot(0, 0);
        });
        imp.header.add_controller(list_drag_source);

        let list_drop_target =
            gtk::DropTarget::new(ProjectList::static_type(), gdk::DragAction::MOVE);
        list_drop_target.set_preload(true);
        list_drop_target.connect_drop(glib::clone!(@weak self as obj => @default-return false,
            move |target, value, x, y| obj.list_drop_target_drop(target, value, x, y)));
        list_drop_target.connect_motion(
            glib::clone!(@weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.list_drop_target_motion(target, x, y)),
        );
        self.add_controller(list_drop_target);
    }

    #[template_callback]
    fn task_activated(&self, row: TaskRow, tasks_box: gtk::ListBox) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = TaskWindow::new(&win.application().unwrap(), &win, row.task());
        modal.present();
        row.cancel_timer();
        modal.connect_closure(
            "task-window-close",
            true,
            glib::closure_local!(@watch row => move |_win: TaskWindow, task: Task| {
                if task.done() {
                    tasks_box.remove(row);
                } else {
                    row.reset(task);
                    row.changed();
                    row.activate_action("project.update", None).expect("Failed to send project.update signal");
                }
            }
        ));
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let name = entry.buffer().text();
        let list = self.list();
        self.imp().name_button.set_visible(true);
        list.set_name(name);
        update_list(&list).expect("Failed to update list");
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        let list = self.list();
        let task = create_task("", list.project(), list.id(), 0).expect("Failed to create task");
        self.imp().tasks_box.add_fresh_task(task);
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        imp.options_button.popdown();
        let dialog = gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/delete_dialog.ui")
            .object::<adw::MessageDialog>("dialog")
            .unwrap();
        dialog.set_transient_for(self.root().and_downcast::<gtk::Window>().as_ref());
        let dialog_heading = gettext("Delete \"{}\" list?");
        dialog.set_heading(Some(&dialog_heading.replace("{}", &self.list().name())));
        dialog.set_body(&gettext("The list and its tasks will be permanently lost."));

        dialog.connect_response(
            Some("delete"),
            glib::clone!(
            @weak self as obj => move |_dialog, response| {
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
                    }}}),
        );
        dialog.present();
    }

    #[template_callback]
    fn handle_show_done_tasks_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        imp.options_button.popdown();
        let win: IPlanWindow = self.root().and_downcast().unwrap();
        let window = ProjectDoneTasksWindow::new(win.application().unwrap(), &win, self.list());
        window.present();
    }

    fn list_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        _value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        // Source list moved by motion signal so it should drop on itself
        let list = self.list();
        let list_db = read_list(list.id()).expect("Failed to read list");
        if list.index() != list_db.index() {
            // TODO: add project condition
            update_list(&list).expect("Failed to update list");
        }
        true
    }

    fn list_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        _y: f64,
    ) -> gdk::DragAction {
        if let Some(source_project_list) = target.value_as::<ProjectList>() {
            let self_list = self.list();
            let source_list = source_project_list.list();
            if self_list.id() != source_list.id() {
                let parent: gtk::Box = self.parent().and_downcast().unwrap();
                let source_i = source_list.index();
                let self_i = self_list.index();
                if source_i - self_i == 1 {
                    parent.reorder_child_after(self, Some(&source_project_list));
                    source_list.set_property("index", self_i);
                    self_list.set_property("index", source_i);
                } else if source_i > self_i {
                    let lists = parent.observe_children();
                    for i in self_i..source_i {
                        let project_list =
                            lists.item(i as u32).and_downcast::<ProjectList>().unwrap();
                        project_list.list().set_property("index", i + 1);
                    }
                    if let Some(upper_list) = lists.item((self_i - 1) as u32) {
                        parent.reorder_child_after(
                            &source_project_list,
                            Some(&upper_list.downcast::<ProjectList>().unwrap()),
                        );
                    } else {
                        parent.reorder_child_after(&source_project_list, gtk::Widget::NONE);
                    }
                    source_list.set_property("index", self_i);
                } else if source_i - self_i == -1 {
                    parent.reorder_child_after(&source_project_list, Some(self));
                    source_list.set_property("index", self_i);
                    self_list.set_property("index", source_i);
                } else if source_i < self_i {
                    //
                    let lists = parent.observe_children();
                    for i in source_i + 1..self_i + 1 {
                        let project_list =
                            lists.item(i as u32).and_downcast::<ProjectList>().unwrap();
                        project_list.list().set_property("index", i - 1);
                    }
                    parent.reorder_child_after(&source_project_list, Some(self));
                    source_list.set_property("index", self_i);
                }
            }
            gdk::DragAction::MOVE
        } else {
            gdk::DragAction::empty()
        }
    }
}
