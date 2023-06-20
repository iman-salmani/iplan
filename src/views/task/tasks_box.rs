use adw::prelude::*;
use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::{gdk, glib, glib::Properties, subclass::prelude::*};
use std::cell::Cell;
use std::thread;
use std::time::Duration;

use crate::db::models::Task;
use crate::db::operations::{create_task, new_position, read_task, update_task};
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
        #[property(get, set=Self::set_scrollable)]
        pub scrollable: Cell<bool>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
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
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                items_wrapper: Cell::new(None),
                scrollable: Cell::new(true),
                scroll: Cell::new(0),
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
            let policy_type = if scrollable {
                gtk::PolicyType::Automatic
            } else {
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
        let row = TaskRow::new(task, false);
        self.imp().items_box.append(&row);
    }

    pub fn add_tasks(&self, tasks: Vec<Task>) {
        let imp = self.imp();
        for task in tasks {
            let row = TaskRow::new(task, false);
            imp.items_box.append(&row);
        }
    }

    pub fn add_fresh_task(&self, task: Task) {
        let row = TaskRow::new(task, false);
        self.imp().items_box.prepend(&row);
        let row_imp = row.imp();
        row_imp.name_button.set_visible(false);
        row_imp.name_entry.grab_focus();
    }

    pub fn add_tasks_lazy(&self, tasks: Vec<Task>, height: usize) {
        let imp = self.imp();
        let page_size = height / 50;
        if tasks.len() > page_size && self.scrollable() {
            let (first_rows, other_rows) = tasks.split_at(page_size); // FIXME: check if height += 50 have a better better performance
            for task in first_rows {
                let task_row = TaskRow::new(task.to_owned(), false);
                imp.items_box.append(&task_row);
            }
            for task in other_rows {
                let task_row = TaskRow::new_lazy(task);
                imp.items_box.append(&task_row);
            }
        } else {
            for task in tasks {
                let task_row = TaskRow::new(task, false);
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
        let index = items.n_items() - 1 - position as u32;
        if let Some(item) = items.item(index) {
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
    }

    pub fn add_item(&self, item: &TaskRow) {
        self.imp().items_box.append(item);
    }

    pub fn send_hscroll(&self) {
        let imp = self.imp();
        let controller = gtk::EventControllerScroll::builder()
            .flags(gtk::EventControllerScrollFlags::VERTICAL)
            .build();
        controller.connect_scroll(
            glib::clone!(@weak self as obj => @default-return gtk::Inhibit(false),
                move |controller, _dx, dy| {
                    if controller.current_event_state().contains(gdk::ModifierType::SHIFT_MASK) {
                        obj.activate_action("hscroll", Some(&dy.to_variant())).unwrap();
                        obj.imp().scrolled_window.vscrollbar().set_sensitive(false);
                        gtk::Inhibit(true)
                    } else {
                        obj.imp().scrolled_window.vscrollbar().set_sensitive(true); // FIXME: Its fine but dont show scrollbar while hovering after scroll ends with sensitive false
                        gtk::Inhibit(false)
                    }
                }
            ),
        );
        imp.scrolled_window.add_controller(controller);
    }

    fn create_empty_task(&self) -> Task {
        match self.items_wrapper().expect("items_wrapper cant be None") {
            TasksBoxWrapper::Section(id, project) => create_task("", project, id, 0).unwrap(),
            TasksBoxWrapper::Task(id, _) => create_task("", 0, 0, id).unwrap(),
            TasksBoxWrapper::Date(date) => {
                let task = create_task("", 1, 0, 0).unwrap();
                task.set_date(date);
                update_task(&task).unwrap();
                task
            }
        }
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
                let first_child = imp.items_box.first_child().unwrap();
                if first_child.widget_name() == "GtkListBoxRow" {
                    imp.bottom_add_task.set_visible(false);
                } else if !imp.bottom_add_task.is_visible() {
                    imp.bottom_add_task.set_visible(true);
                } else {
                    let row = first_child.downcast::<TaskRow>().unwrap();
                    if row.task().suspended() || row.moving_out() {
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

        let mut possible_row = row.and_upcast::<gtk::Widget>();
        loop {
            if let Some(row) = possible_row {
                if let Some(row) = row.downcast_ref::<TaskRow>() {
                    if row.lazy() {
                        row.set_lazy(false);
                        possible_row = row.prev_sibling();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
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
        // Source row moved by motion signal so it should drop on itself
        let imp = self.imp();
        imp.items_box.drag_unhighlight_row();
        imp.items_box.set_height_request(-1);
        let row: TaskRow = value.get().unwrap();
        let task = row.task();
        let task_db = read_task(task.id()).expect("Failed to read task");
        if let TasksBoxWrapper::Date(_) = self.items_wrapper().unwrap() {
            update_task(&task).expect("Failed to update task");
        } else if task_db.position() != task.position() || task_db.section() != task.section() {
            update_task(&task).expect("Failed to update task");
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

        let source_row: Option<TaskRow> = target.value_as();
        let target_row = imp.items_box.row_at_y(y as i32);

        if self.imp().items_box.observe_children().n_items() == 2 {
            return gdk::DragAction::MOVE;
        } else if source_row.is_none() || target_row.is_none() {
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
            let source_i = source_row.index();
            let target_i = target_row.index();
            let source_p = source_task.position();
            let target_p = target_task.position();
            if source_i - target_i == 1 {
                source_task.set_property("position", source_p + 1);
                target_task.set_property("position", target_p - 1);
            } else if source_i > target_i {
                for i in target_i..source_i {
                    let row: TaskRow = imp.items_box.row_at_index(i).and_downcast().unwrap();
                    row.task()
                        .set_property("position", row.task().position() - 1);
                }
                source_task.set_property("position", target_p)
            } else if source_i - target_i == -1 {
                source_task.set_property("position", source_p - 1);
                target_task.set_property("position", target_p + 1);
            } else if source_i < target_i {
                for i in source_i + 1..target_i + 1 {
                    let row: TaskRow = imp.items_box.row_at_index(i).and_downcast().unwrap();
                    row.task()
                        .set_property("position", row.task().position() + 1);
                }
                source_task.set_property("position", target_p)
            }

            // Should use invalidate_sort() insteed of changed() for refresh highlight shape
            imp.items_box.invalidate_sort();
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
                task.set_position(new_position(section_id));
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
                if imp.items_box.observe_children().n_items() == 2 {
                    imp.items_box.set_height_request(320);
                }
                imp.items_box.prepend(&row);
                imp.items_box.drag_highlight_row(&row);
            }
            if let TasksBoxWrapper::Date(date) = items_wrapper {
                row.set_moving_out(false);
                let task = row.task();
                task.set_date(date);
                let parent = row.parent().and_downcast::<gtk::ListBox>().unwrap();
                parent.remove(&row);
                if imp.items_box.observe_children().n_items() == 2 {
                    imp.items_box.set_height_request(320);
                }
                imp.items_box.prepend(&row);
                imp.items_box.drag_highlight_row(&row);
                row.reset(task);
            }
        }
        gdk::DragAction::MOVE
    }

    fn task_drop_target_leave(&self, target: &gtk::DropTarget) {
        if let Some(row) = target.value_as::<TaskRow>() {
            row.set_moving_out(true);
            let items_box: &gtk::ListBox = self.imp().items_box.as_ref();
            items_box.invalidate_filter();
            items_box.set_height_request(-1);
        }
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
        let task_ui = TaskRow::new(task, false);
        let imp = self.imp();
        imp.items_box.prepend(&task_ui);
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
    }

    #[template_callback]
    fn new_task_bottom(&self, _button: gtk::Button) {
        let imp = self.imp();
        let task = self.create_empty_task();

        task.set_position(0);
        let task_rows = imp.items_box.observe_children();
        for i in 0..task_rows.n_items() {
            if let Some(row) = task_rows.item(i as u32).and_downcast::<TaskRow>() {
                let row_task = row.task();
                row_task.set_position(row_task.position() + 1);
            }
        }

        update_task(&task).unwrap();

        let task_ui = TaskRow::new(task, false);
        imp.items_box.append(&task_ui);
        let task_imp = task_ui.imp();
        task_imp.name_button.set_visible(false);
        task_imp.name_entry.grab_focus();
        let vadjustment = imp.scrolled_window.vadjustment();
        vadjustment.set_value(vadjustment.upper());
    }
}
