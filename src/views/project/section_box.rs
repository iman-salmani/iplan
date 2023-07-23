use adw::prelude::*;
use gettextrs::gettext;
use gtk::{gdk, glib, glib::Properties, subclass::prelude::*};
use std::cell::RefCell;

use crate::db::models::{Section, Task};
use crate::db::operations::{
    create_task, delete_section, new_task_position, read_section, read_tasks, update_section,
};
use crate::views::project::ProjectLayout;
use crate::views::snippets::MenuItem;
use crate::views::task::{TaskRow, TaskWindow, TasksBox, TasksBoxWrapper, TasksDoneWindow};
use crate::views::{ActionScope, IPlanWindow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/section_box.ui")]
    #[properties(wrapper_type=super::SectionBox)]
    pub struct SectionBox {
        #[property(get, set)]
        pub section: RefCell<Section>,
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
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SectionBox {
        const NAME: &'static str = "SectionBox";
        type Type = super::SectionBox;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            MenuItem::ensure_type();
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action(
                "task.changed",
                Some(Task::static_variant_type().as_str()),
                move |obj, _, value| {
                    let imp = obj.imp();
                    let task: Task = value.unwrap().get().unwrap();

                    obj.activate_task_action("task.changed", &task);

                    if !task.done() {
                        return;
                    }

                    let row = imp.tasks_box.item_by_id(task.id()).unwrap();
                    let index = row.index() as u32;
                    if index != 0 {
                        let upper_row = imp.tasks_box.item_by_index(index - 1);
                        if let Some(upper_row) = upper_row {
                            upper_row.grab_focus();
                        }
                    }
                    imp.tasks_box.remove_item(&row);

                    let mut toast_name = task.name();
                    if let Some((i, _)) = toast_name.char_indices().nth(14) {
                        toast_name.truncate(i);
                        toast_name.push_str("...");
                    }
                    let toast = adw::Toast::builder()
                        .title(
                            gettext("“{}” moved to the done tasks list").replace("{}", &toast_name),
                        )
                        .button_label(gettext("Undo"))
                        .build();
                    toast.connect_button_clicked(
                        glib::clone!(@weak obj, @strong row => move |_toast| {
                            obj.imp().tasks_box.add_item(&row);
                            row.imp().checkbox.set_active(false);
                        }),
                    );
                    let window = obj.root().and_downcast::<IPlanWindow>().unwrap();
                    window.imp().toast_overlay.add_toast(toast);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SectionBox {
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
    impl WidgetImpl for SectionBox {}
    impl BoxImpl for SectionBox {}
}

glib::wrapper! {
    pub struct SectionBox(ObjectSubclass<imp::SectionBox>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Buildable;
}

#[gtk::template_callbacks]
impl SectionBox {
    pub fn new(section: Section, layout: ProjectLayout, max_height: usize) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.set_section(section);
        let imp = obj.imp();
        let section = obj.section();

        imp.name_entry.buffer().set_text(&section.name());

        let tasks = read_tasks(
            Some(section.project()),
            Some(section.id()),
            Some(false),
            Some(0),
            None,
            false,
        )
        .unwrap();

        obj.set_layout(&layout);
        imp.tasks_box
            .set_items_wrapper(TasksBoxWrapper::Section(section.id(), section.project()));
        imp.tasks_box.add_tasks_lazy(tasks, max_height);

        obj
    }

    pub fn set_layout(&self, layout: &ProjectLayout) {
        let imp = self.imp();
        if layout == &ProjectLayout::Horizontal {
            imp.tasks_box.set_scrollable(true);
        } else {
            imp.tasks_box.set_scrollable(false);
            let mut lazy_tasks = imp.tasks_box.imp().lazy_tasks.borrow_mut();
            for _ in 0..lazy_tasks.len() {
                imp.tasks_box.add_task(lazy_tasks.pop().unwrap());
            }
        }
    }

    pub fn select_task(&self, target_task: Task) {
        let imp = self.imp();
        let task_rows = imp.tasks_box.observe_children();
        for i in 0..task_rows.n_items() - 1 {
            if let Some(task_row) = task_rows.item(i).and_downcast::<TaskRow>() {
                let section_task = task_row.task();
                if section_task.position() == target_task.position() {
                    task_row.grab_focus();
                    break;
                }
            }
        }
    }

    fn add_drag_drop_controllers(&self) {
        let imp = self.imp();
        let section_drag_source = gtk::DragSource::builder()
            .actions(gdk::DragAction::MOVE)
            .build();
        section_drag_source.connect_prepare(
            glib::clone!(@weak self as obj => @default-return None,
            move |_drag_source, _x, _y| {
                if obj.imp().name_entry.get_visible() {
                    None
                } else {
                    Some(gdk::ContentProvider::for_value(&obj.to_value()))
                }
            }),
        );
        section_drag_source.connect_drag_begin(|_drag_source, drag| {
            let drag_icon: gtk::DragIcon = gtk::DragIcon::for_drag(drag).downcast().unwrap();
            let label = gtk::Label::builder().label("").build();
            drag_icon.set_child(Some(&label));
            drag.set_hotspot(0, 0);
        });
        imp.header.add_controller(section_drag_source);

        let section_drop_target =
            gtk::DropTarget::new(SectionBox::static_type(), gdk::DragAction::MOVE);
        section_drop_target.set_preload(true);
        section_drop_target.connect_drop(glib::clone!(@weak self as obj => @default-return false,
            move |target, value, x, y| obj.section_drop_target_drop(target, value, x, y)));
        section_drop_target.connect_motion(
            glib::clone!(@weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.section_drop_target_motion(target, x, y)),
        );
        self.add_controller(section_drop_target);
    }

    fn activate_task_action(&self, name: &str, task: &Task) {
        let project_id = self.section().project();
        self.parent()
            .unwrap()
            .activate_action(
                name,
                Some(&glib::Variant::from((
                    task.to_variant(),
                    ActionScope::Project(project_id).to_variant(),
                ))),
            )
            .unwrap();
    }

    #[template_callback]
    fn task_activated(&self, row: TaskRow, tasks_box: gtk::ListBox) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let modal = TaskWindow::new(&win.application().unwrap(), &win, row.task());
        modal.present();
        modal.connect_close_request(
            glib::clone!(@weak row => @default-return gtk::Inhibit(false), move |_| {
                let task = row.task();
                if task.done() {
                    tasks_box.remove(&row);
                } else if task.suspended() {
                    row.changed();
                }
                gtk::Inhibit(false)
            }),
        );
        modal.connect_closure(
            "task-changed",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, changed_task: Task| {
                let row = row.unwrap();
                let task = row.task();
                let task_id = task.id();
                obj.activate_task_action("task.changed", &changed_task);
                if task_id == changed_task.id() {
                    row.reset(changed_task);
                } else if task_id == changed_task.parent() {
                    row.reset_subtasks();
                }
            }),
        );
        modal.connect_closure(
            "task-duration-changed",
            true,
            glib::closure_local!(@watch self as obj, @weak-allow-none row => move |_win: TaskWindow, task: Task| {
                let row = row.unwrap();
                obj.activate_action("task.duration-changed", Some(&task.to_variant())).unwrap();
                row.refresh_timer();
            }),
        );
    }

    #[template_callback]
    fn handle_name_button_clicked(&self, button: gtk::Button) {
        button.set_visible(false); // Entry visible param binded to this
        self.imp().name_entry.grab_focus_without_selecting();
    }

    #[template_callback]
    fn handle_name_entry_activate(&self, entry: gtk::Entry) {
        let name = entry.buffer().text();
        let section = self.section();
        self.imp().name_button.set_visible(true);
        section.set_name(name);
        update_section(&section).expect("Failed to update section");
    }

    #[template_callback]
    fn handle_new_button_clicked(&self, _button: gtk::Button) {
        let section = self.section();
        let section_id = section.id();
        let task = create_task(Task::new(&[
            ("project", &section.project()),
            ("section", &section_id),
            ("position", &new_task_position(section_id)),
        ]))
        .unwrap();
        self.imp().tasks_box.add_fresh_task(task);
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        imp.options_button.popdown();
        let dialog =
            gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/snippets/delete_dialog.ui")
                .object::<adw::MessageDialog>("dialog")
                .unwrap();
        dialog.set_transient_for(self.root().and_downcast::<gtk::Window>().as_ref());
        let dialog_heading = gettext("Delete “{}” section?");
        dialog.set_heading(Some(&dialog_heading.replace("{}", &self.section().name())));
        dialog.set_body(&gettext(
            "The section and its tasks will be permanently lost.",
        ));

        dialog.connect_response(
            Some("delete"),
            glib::clone!(
            @weak self as obj => move |_dialog, response| {
                if response == "delete" {
                    delete_section(obj.section().id()).expect("Failed to delete section");
                    let sections_box = obj.parent().and_downcast::<gtk::Box>().unwrap();
                    let placeholder = obj.root()
                        .and_downcast::<IPlanWindow>()
                        .unwrap()
                        .visible_project_page()
                        .unwrap()
                        .imp()
                        .placeholder
                        .get();
                    sections_box.remove(&obj);
                    if sections_box.first_child().is_none() {
                        sections_box.append(&placeholder);
                    }}}),
        );
        dialog.present();
    }

    #[template_callback]
    fn handle_show_done_tasks_button_clicked(&self, _button: gtk::Button) {
        let imp = self.imp();
        imp.options_button.popdown();
        let win: IPlanWindow = self.root().and_downcast().unwrap();
        let window = TasksDoneWindow::new(win.application().unwrap(), &win, self.section());
        window.present();
        window.connect_closure(
            "task-undo",
            true,
            glib::closure_local!(@watch self as obj => move |_: TasksDoneWindow, task: Task| {
                let imp = obj.imp();
                imp.tasks_box.add_task(task);
            }),
        );
    }

    fn section_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        _value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        // Source section moved by motion signal so it should drop on itself
        let section = self.section();
        let section_db = read_section(section.id()).expect("Failed to read section");
        if section.index() != section_db.index() {
            // TODO: add project condition
            update_section(&section).expect("Failed to update section");
        }
        true
    }

    fn section_drop_target_motion(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        _y: f64,
    ) -> gdk::DragAction {
        if let Some(source_project_section) = target.value_as::<SectionBox>() {
            let self_section = self.section();
            let source_section = source_project_section.section();
            if self_section.id() != source_section.id() {
                let parent: gtk::Box = self.parent().and_downcast().unwrap();
                let source_i = source_section.index();
                let self_i = self_section.index();
                if source_i - self_i == 1 {
                    parent.reorder_child_after(self, Some(&source_project_section));
                    source_section.set_property("index", self_i);
                    self_section.set_property("index", source_i);
                } else if source_i > self_i {
                    let sections = parent.observe_children();
                    for i in self_i..source_i {
                        let project_section = sections
                            .item(i as u32)
                            .and_downcast::<SectionBox>()
                            .unwrap();
                        project_section.section().set_property("index", i + 1);
                    }
                    if let Some(upper_section) = sections.item((self_i - 1) as u32) {
                        parent.reorder_child_after(
                            &source_project_section,
                            Some(&upper_section.downcast::<SectionBox>().unwrap()),
                        );
                    } else {
                        parent.reorder_child_after(&source_project_section, gtk::Widget::NONE);
                    }
                    source_section.set_property("index", self_i);
                } else if source_i - self_i == -1 {
                    parent.reorder_child_after(&source_project_section, Some(self));
                    source_section.set_property("index", self_i);
                    self_section.set_property("index", source_i);
                } else if source_i < self_i {
                    //
                    let sections = parent.observe_children();
                    for i in source_i + 1..self_i + 1 {
                        let project_section = sections
                            .item(i as u32)
                            .and_downcast::<SectionBox>()
                            .unwrap();
                        project_section.section().set_property("index", i - 1);
                    }
                    parent.reorder_child_after(&source_project_section, Some(self));
                    source_section.set_property("index", self_i);
                }
            }
            gdk::DragAction::MOVE
        } else {
            gdk::DragAction::empty()
        }
    }
}
