use adw::prelude::*;
use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{gdk, glib, glib::Properties, graphene, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::thread;
use std::time::Duration;

use crate::db::models::Task;
use crate::db::operations::{
    create_task, new_subtask_position, new_task_position, read_task, update_task,
};
use crate::views::task::TaskRow;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TasksBoxWrapper {
    Section(i64, i64),
    Task(i64, i64),
    Date(i64),
}

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/task/tasks_box.ui")]
    #[properties(wrapper_type=super::TasksBox)]
    pub struct TasksBox {
        pub items_wrapper: Cell<Option<TasksBoxWrapper>>,
        pub lazy_tasks: RefCell<Vec<Task>>,
        #[property(get, set=Self::set_scrollable)]
        pub scrollable: Cell<bool>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
        #[property(get, set)]
        pub hscroll_controller: RefCell<Option<gtk::EventControllerScroll>>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub items_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub bottom_add_task: TemplateChild<gtk::ListBoxRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TasksBox {
        const NAME: &'static str = "TasksBox";
        type Type = super::TasksBox;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.move-up", Some("x"), move |obj, _, value| {
                let id: i64 = value.unwrap().get().unwrap();
                obj.move_item_one_step(id, true);
            });
            klass.install_action("task.move-down", Some("x"), move |obj, _, value| {
                let id: i64 = value.unwrap().get().unwrap();
                obj.move_item_one_step(id, false);
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                items_wrapper: Cell::new(None),
                lazy_tasks: RefCell::new(vec![]),
                scrollable: Cell::new(true),
                scroll: Cell::new(0),
                hscroll_controller: RefCell::new(None),
                scrolled_window: gtk::TemplateChild::default(),
                items_box: gtk::TemplateChild::default(),
                bottom_add_task: gtk::TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for TasksBox {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.init_scroller();
            obj.set_items_box_funcs();
            obj.add_drag_drop_controllers();
        }

        fn dispose(&self) {
            let obj = self.obj();
            obj.imp().scrolled_window.unparent();
        }

        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("task-activated")
                    .param_types([TaskRow::static_type(), gtk::ListBox::static_type()])
                    .build()]
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
    impl WidgetImpl for TasksBox {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.parent_request_mode();
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.obj()
                .first_child()
                .unwrap()
                .measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.obj()
                .first_child()
                .unwrap()
                .size_allocate(&gtk::Allocation::new(0, 0, width, height), baseline);
        }

        fn map(&self) {
            self.parent_map();
            let obj = self.obj();
            if let Some(win) = obj.root().and_downcast::<gtk::Window>() {
                win.connect_default_height_notify(glib::clone!(@weak obj => move |_| {
                    obj.load_lazy_rows();
                }));
            }
        }
    }

    impl TasksBox {
        pub fn set_scrollable(&self, scrollable: bool) {
            let obj = self.obj();
            let policy_type = if scrollable {
                obj.send_hscroll();
                gtk::PolicyType::Automatic
            } else {
                obj.disable_hscroll();
                gtk::PolicyType::Never
            };
            self.scrolled_window.set_vscrollbar_policy(policy_type);
            self.scrollable.set(scrollable);
        }
    }
}

glib::wrapper! {
    pub struct TasksBox(ObjectSubclass<imp::TasksBox>)
        @extends gtk::Widget,
        @implements gtk::Buildable;
}

impl Default for TasksBox {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl TasksBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_task(&self, task: Task) {
        let row = self.create_task_row(task);
        self.imp().items_box.append(&row);
    }

    pub fn add_tasks(&self, tasks: Vec<Task>) {
        let imp = self.imp();
        for task in tasks {
            let row = self.create_task_row(task);
            imp.items_box.append(&row);
        }
    }

    pub fn add_fresh_task(&self, task: Task) {
        let row = self.create_task_row(task);
        self.imp().items_box.prepend(&row);
        let row_imp = row.imp();
        row_imp.name_button.set_visible(false);
        row_imp.name_entry.grab_focus();
    }

    pub fn add_tasks_lazy(&self, mut tasks: Vec<Task>, height: usize) {
        let imp = self.imp();
        let page_tasks_count = height / 50;
        if tasks.len() > page_tasks_count && self.scrollable() {
            for _ in 0..page_tasks_count {
                let task = tasks.pop().unwrap();
                let task_row = self.create_task_row(task);
                imp.items_box.append(&task_row);
            }
            imp.lazy_tasks.replace(tasks);
        } else {
            for task in tasks {
                let task_row = self.create_task_row(task);
                imp.items_box.append(&task_row);
            }
        }
    }

    pub fn items_wrapper(&self) -> Option<TasksBoxWrapper> {
        self.imp().items_wrapper.get()
    }

    pub fn set_items_wrapper(&self, wrapper: TasksBoxWrapper) {
        self.imp().items_wrapper.replace(Some(wrapper));
    }

    pub fn item_by_index(&self, index: u32) -> Option<TaskRow> {
        let imp = self.imp();
        let items = imp.items_box.observe_children();
        if let Some(item) = items.item(index) {
            if let Ok(task_row) = item.downcast::<TaskRow>() {
                return Some(task_row);
            }
        }
        None
    }

    pub fn item_by_position(&self, position: i32) -> Option<TaskRow> {
        let imp = self.imp();
        let items = imp.items_box.observe_children();
        let index = items.n_items() as i32 - 1 - position;
        if let Some(item) = items.item(index as u32) {
            if let Ok(task_row) = item.downcast::<TaskRow>() {
                return Some(task_row);
            }
        }
        None
    }

    pub fn item_by_id(&self, id: i64) -> Option<TaskRow> {
        let imp = self.imp();
        let items = imp.items_box.observe_children();
        for i in 0..items.n_items() {
            if let Some(row) = items.item(i).and_downcast::<TaskRow>() {
                if row.task().id() == id {
                    return Some(row);
                }
            }
        }
        None
    }

    pub fn remove_item(&self, item: &TaskRow) {
        self.imp().items_box.remove(item);
        self.imp().items_box.invalidate_filter();
    }

    pub fn add_item(&self, item: &TaskRow) {
        self.imp().items_box.append(item);
        self.imp().items_box.invalidate_filter();
    }

    fn send_hscroll(&self) {
        let imp = self.imp();
        let controller = self.hscroll_controller().unwrap_or_else(|| {
            let controller = gtk::EventControllerScroll::builder()
                .flags(gtk::EventControllerScrollFlags::VERTICAL)
                .build();
            controller.connect_scroll(
                glib::clone!(@weak self as obj => @default-return gtk::Inhibit(false), move |controller, _dx, dy| {
                    if controller.current_event_state().contains(gdk::ModifierType::SHIFT_MASK) {
                        obj.activate_action("hscroll", Some(&dy.to_variant())).unwrap();
                        obj.imp().scrolled_window.vscrollbar().set_sensitive(false);
                        gtk::Inhibit(true)
                    } else {
                        obj.imp().scrolled_window.vscrollbar().set_sensitive(true); // FIXME: Its fine but dont show scrollbar while hovering after scroll ends with sensitive false
                        gtk::Inhibit(false)
                    }
                }),
            );

            self.set_hscroll_controller(&controller);
            controller
        });
        imp.scrolled_window.add_controller(controller);
    }

    fn disable_hscroll(&self) {
        let imp = self.imp();
        if let Some(controller) = self.hscroll_controller() {
            imp.scrolled_window.remove_controller(&controller);
        }
    }

    fn create_empty_task(&self) -> Task {
        match self.items_wrapper().expect("items_wrapper cant be None") {
            TasksBoxWrapper::Section(id, project) => create_task(Task::new(&[
                ("project", &project),
                ("section", &id),
                ("position", &new_task_position(id)),
            ]))
            .unwrap(),
            TasksBoxWrapper::Task(id, project) => create_task(Task::new(&[
                ("project", &project),
                ("parent", &id),
                ("position", &new_subtask_position(id)),
            ]))
            .unwrap(),
            TasksBoxWrapper::Date(date) => create_task(Task::new(&[("date", &date)])).unwrap(),
        }
    }

    fn create_task_row(&self, task: Task) -> TaskRow {
        let visible_project_label = if let TasksBoxWrapper::Date(_) = self.items_wrapper().unwrap()
        {
            true
        } else {
            false
        };
        let row = TaskRow::new(task, false, visible_project_label);
        if let TasksBoxWrapper::Date(_) = self.items_wrapper().unwrap() {
            row.set_hide_move_arrows(true);
        }
        row
    }

    fn set_items_box_funcs(&self) {
        let imp = self.imp();

        imp.items_box.set_sort_func(glib::clone!(@weak self as obj => @default-return gtk::Ordering::Larger, move |row1, row2| {
            if let Some(wrapper) = obj.items_wrapper() {
                if let TasksBoxWrapper::Date(_) = wrapper {
                    return gtk::Ordering::Larger;
                }
            }
            let row1 = if let Some(row1) = row1.downcast_ref::<TaskRow>() {
                row1
            } else {
                return gtk::Ordering::Larger;
            };
            let row2 = if let Some(row2) = row2.downcast_ref::<TaskRow>() {
                row2
            } else {
                return gtk::Ordering::Smaller;
            };

            if row1.task().position() < row2.task().position() {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        }));

        imp.items_box.set_filter_func(
            glib::clone!(@weak self as obj => @default-return false, move |row| {
                let imp = obj.imp();

                let items = imp.items_box.observe_children();
                let items_len = items.n_items();
                if items_len == 2 {
                    imp.bottom_add_task.set_visible(false);
                } else {
                    let mut visible_task_row = false;
                    for i in 0..items_len {
                        if let Some(task_row) = items.item(i).and_downcast::<TaskRow>() {
                            if !task_row.moving_out() && !task_row.task().suspended() {
                                visible_task_row = true;
                                break;
                            }
                        }
                    }
                    if visible_task_row {
                        imp.bottom_add_task.set_visible(true);
                    } else {
                        imp.bottom_add_task.set_visible(false);
                    }
                }

                if let Some(row) = row.downcast_ref::<TaskRow>() {
                    if row.task().suspended() {
                        false
                    } else {
                        !row.moving_out()
                    }
                } else {
                    true
                }
            }),
        );
    }

    fn init_scroller(&self) {
        let imp = self.imp();

        imp.scrolled_window
            .vadjustment()
            .connect_value_changed(glib::clone!(@weak self as obj =>
                move |_| {
                    obj.load_lazy_rows();
                }
            ));
    }

    fn load_lazy_rows(&self) {
        let imp = self.imp();
        let scrolled_win = imp.scrolled_window.get();
        let pos = scrolled_win.vadjustment().value();
        let height = imp.scrolled_window.height();
        let row = imp.items_box.row_at_y(height + pos as i32 - 50);

        let mut lazy_tasks = imp.lazy_tasks.borrow_mut();
        if lazy_tasks.is_empty() {
            return;
        }

        let rows = imp.items_box.observe_children();
        let last_row = if let Some(row) = rows.item(rows.n_items() - 3) {
            row.downcast::<TaskRow>().unwrap()
        } else {
            return;
        };

        let possible_row = row.and_upcast::<gtk::Widget>();
        if let Some(row) = possible_row {
            if let Ok(row) = row.downcast::<TaskRow>() {
                let difference = last_row.index() - row.index();
                if difference < 5 {
                    for _ in 0..difference {
                        if let Some(row) = lazy_tasks.pop() {
                            let new_row = self.create_task_row(row);
                            imp.items_box.append(&new_row);
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    fn add_drag_drop_controllers(&self) {
        let imp = self.imp();

        let task_drop_target = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        task_drop_target.set_preload(true);
        task_drop_target.connect_drop(glib::clone!(@weak self as obj => @default-return false,
            move |target, value, x, y| obj.task_drop_target_drop(target, value, x, y)));
        task_drop_target.connect_motion(
            glib::clone!(@weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_motion(target, x, y)),
        );
        task_drop_target.connect_enter(
            glib::clone!(@weak self as obj => @default-return gdk::DragAction::empty(),
            move |target, x, y| obj.task_drop_target_enter(target, x, y)),
        );
        task_drop_target.connect_leave(glib::clone!(@weak self as obj =>
            move |target| obj.task_drop_target_leave(target)));
        imp.items_box.add_controller(task_drop_target);
    }

    fn task_drop_target_drop(
        &self,
        _target: &gtk::DropTarget,
        value: &glib::Value,
        _x: f64,
        _y: f64,
    ) -> bool {
        let imp = self.imp();
        imp.items_box.drag_unhighlight_row();
        let row: TaskRow = value.get().unwrap();
        let task = row.task();
        let task_db = read_task(task.id()).unwrap();
        let items_wrapper = self.items_wrapper().unwrap();

        if let TasksBoxWrapper::Date(_) = items_wrapper {
            update_task(&task).unwrap();
        } else if row.moving_out() {
            let task_parent = task.parent();
            task.set_section(0);
            task.set_position(new_subtask_position(task_parent));
            update_task(&task).unwrap();
            imp.items_box.remove(&row);
            if let Some(parent_row) = self.item_by_id(task_parent) {
                parent_row.add_subtask(task);
                parent_row.reset_timer();
                parent_row.imp().subtask_drop_target.set_visible(false);
            }
        } else if task_db.position() != task.position() || task_db.section() != task.section() {
            update_task(&task).unwrap();
        }
        row.grab_focus();

        if let TasksBoxWrapper::Task(_, _) = items_wrapper {
            self.activate_action("task.changed", Some(&task_db.to_variant()))
                .unwrap();
        }

        true
    }

    fn task_drop_target_motion(&self, target: &gtk::DropTarget, x: f64, y: f64) -> gdk::DragAction {
        let imp = self.imp();

        let source_row: Option<TaskRow> = target.value_as();
        let target_row = imp.items_box.row_at_y(y as i32);
        if source_row.is_none() || target_row.is_none() {
            return gdk::DragAction::empty();
        }
        let source_row = source_row.unwrap();
        let target_row: TaskRow = if let Some(row) = target_row.and_downcast() {
            row
        } else {
            return gdk::DragAction::empty();
        };

        if let TasksBoxWrapper::Date(_) = self.items_wrapper().unwrap() {
            return gdk::DragAction::MOVE;
        }

        // Move
        let source_task = source_row.task();
        let target_task = target_row.task();
        if source_task.id() != target_task.id() {
            let point_on_target = imp
                .items_box
                .compute_point(&target_row, &graphene::Point::new(x as f32, y as f32))
                .unwrap();
            let move_task = |step: i32, order: Ordering| {
                let target_i = target_row.index();
                let source_i: i32 = source_row.index();
                let source_p = source_task.position();
                let target_p = target_task.position();
                source_row.set_moving_out(false);
                source_task.set_parent(target_task.parent());

                for i in (target_i - 2)..(target_i + 2) {
                    if let Some(row) = imp.items_box.row_at_index(i) {
                        if let Ok(task_row) = row.downcast::<TaskRow>() {
                            task_row.imp().subtask_drop_target.set_visible(false);
                        }
                    }
                }
                if source_i - target_i == step {
                    source_task.set_position(source_p + step);
                    target_task.set_position(target_p - step);
                } else if source_i.cmp(&target_i) == order {
                    for i in target_i..source_i {
                        let row: TaskRow = imp.items_box.row_at_index(i).and_downcast().unwrap();
                        row.task().set_position(row.task().position() - 1);
                    }
                    source_task.set_position(target_p)
                }
            };
            if point_on_target.y() <= 6.0 {
                move_task(1, Ordering::Greater);
            } else if target_row.height() - point_on_target.y() as i32 <= 6 {
                move_task(-1, Ordering::Less);
            } else if source_task.parent() != target_task.id() {
                target_row.imp().subtask_drop_target.set_visible(true);
                source_task.set_parent(target_task.id());
                source_row.set_moving_out(true);
            }
            source_row.changed();
        }

        // Scroll
        if self.scrollable() {
            let scrolled_window_height = imp.scrolled_window.height();
            if imp.items_box.height() > scrolled_window_height {
                let adjustment = imp.scrolled_window.vadjustment();
                let v_pos = adjustment.value();
                if y - v_pos > (scrolled_window_height - 50) as f64 {
                    if self.scroll() != 1 {
                        self.set_scroll(1);
                        self.start_scroll();
                    }
                } else if y - v_pos < 50.0 {
                    if self.scroll() != -1 {
                        self.set_scroll(-1);
                        self.start_scroll();
                    }
                } else {
                    self.set_scroll(0)
                }
            }
        }

        gdk::DragAction::MOVE
    }

    fn task_drop_target_enter(
        &self,
        target: &gtk::DropTarget,
        _x: f64,
        _y: f64,
    ) -> gdk::DragAction {
        let row: TaskRow = target.value_as().unwrap();
        let imp = self.imp();

        let items_wrapper = self.items_wrapper().expect("items_wrapper cant be None");
        let is_same_box = match items_wrapper {
            TasksBoxWrapper::Section(id, _) => row.task().section() == id,
            TasksBoxWrapper::Task(id, _) => row.task().parent() == id,
            TasksBoxWrapper::Date(date) => row.task().date() == date,
        };
        // Check moving_out to Avoid running at drag start
        if is_same_box && row.moving_out() {
            row.set_moving_out(false);
            imp.items_box.invalidate_filter();
        } else if !is_same_box {
            if let TasksBoxWrapper::Section(section_id, _) = items_wrapper {
                row.set_moving_out(false);
                let task = row.task();
                task.set_section(section_id);
                task.set_position(new_task_position(section_id));
                let parent = row.parent().and_downcast::<gtk::ListBox>().unwrap();
                for i in 0..row.index() {
                    let task = parent
                        .row_at_index(i)
                        .and_downcast::<TaskRow>()
                        .unwrap()
                        .task();
                    task.set_position(task.position() - 1);
                }
                parent.remove(&row);
                imp.items_box.prepend(&row);
            }
            if let TasksBoxWrapper::Date(date) = items_wrapper {
                row.set_moving_out(false);
                let task = row.task();
                task.set_date(date);
                let parent = row.parent().and_downcast::<gtk::ListBox>().unwrap();
                parent.remove(&row);
                imp.items_box.prepend(&row);
                row.reset(task);
            }
        }
        gdk::DragAction::MOVE
    }

    fn task_drop_target_leave(&self, target: &gtk::DropTarget) {
        self.set_scroll(0);
        if let Some(row) = target.value_as::<TaskRow>() {
            if let Some(parent_row) = self.item_by_id(row.task().parent()) {
                parent_row.imp().subtask_drop_target.set_visible(false);
            }
            if row.has_css_class("dragged") {
                row.set_moving_out(true);
                row.changed();
            }
        }
    }

    fn move_item_one_step(&self, id: i64, up: bool) {
        let step = if up { -1 } else { 1 };
        let item = self.item_by_id(id).unwrap();
        let item_index = item.index();
        let item_task: Task = item.task();
        let item_task_position = item_task.position();

        if up && item_index == 0 {
            return;
        } else if !up && item_task_position == 0 {
            return;
        }

        let target_item = self.item_by_index((item_index + step) as u32).unwrap();
        let target_item_task = target_item.task();
        let target_item_task_position = target_item_task.position();

        target_item_task.set_position(item_task_position);
        item_task.set_position(target_item_task_position);
        self.imp().items_box.invalidate_sort();
        update_task(&item_task).unwrap();
    }

    fn start_scroll(&self) {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || loop {
            if tx.send(()).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.1));
        });
        rx.attach(
            None,
            glib::clone!(@weak self as obj => @default-return glib::Continue(false), move |_| {
                let scroll = obj.scroll();
                if scroll == 0 {
                    glib::Continue(false)
                } else if scroll.is_positive() {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepDown, false);
                    glib::Continue(true)
                } else {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepUp, false);
                    glib::Continue(true)
                }
            }),
        );
    }

    #[template_callback]
    fn task_activated(&self, item: gtk::ListBoxRow, items_box: gtk::ListBox) {
        let task_row = item.downcast::<TaskRow>().unwrap();
        self.emit_by_name::<()>("task-activated", &[&task_row, &items_box]);
    }

    #[template_callback]
    fn new_task(&self, _button: gtk::Button) {
        let task = self.create_empty_task();
        let task_row = self.create_task_row(task);
        let imp = self.imp();
        imp.items_box.prepend(&task_row);
        let task_imp = task_row.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }

    #[template_callback]
    fn new_task_bottom(&self, _button: gtk::Button) {
        let imp = self.imp();

        let task_rows = imp.items_box.observe_children();
        for i in 0..task_rows.n_items() {
            if let Some(row) = task_rows.item(i as u32).and_downcast::<TaskRow>() {
                let row_task = row.task();
                row_task.set_position(row_task.position() + 1);
            }
        }

        let task = self.create_empty_task();
        task.set_position(0);
        update_task(&task).unwrap();

        if let TasksBoxWrapper::Task(_, _) = self.items_wrapper().unwrap() {
            self.activate_action("task.changed", Some(&task.to_variant()))
                .unwrap();
        }

        let task_row = self.create_task_row(task);
        imp.items_box.append(&task_row);
        let task_imp = task_row.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
        let vadjustment = imp.scrolled_window.vadjustment();
        vadjustment.set_value(vadjustment.upper());
    }
}
