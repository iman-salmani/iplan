import gi
from gi.repository import Gtk, GLib, Gdk

from iplan.db.models.list import List
from iplan.db.operations.list import update_list
from iplan.db.operations.task import create_task, read_tasks, read_task, update_task, find_new_task_position

from iplan.views.project.project_list_task import ProjectListTask
from iplan.views.project.project_list_delete_dialog import ProjectListDeleteDialog


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/project/project_list.ui')
class ProjectList(Gtk.Box):
    __gtype_name__ = "ProjectList"
    _list: List
    scrolled_window: Gtk.ScrolledWindow = Gtk.Template.Child()
    tasks_box: Gtk.ListBox = Gtk.Template.Child()
    name_button: Gtk.Button = Gtk.Template.Child()
    name_entry: Gtk.Entry = Gtk.Template.Child()
    options_button: Gtk.MenuButton = Gtk.Template.Child()
    show_done_tasks_toggle_button: Gtk.ToggleButton = Gtk.Template.Child()
    contain_done_tasks = False
    scroll_controller = None

    def __init__(self, _list: List) -> None:
        self.install_action("task.done", 'i', self.task_done_cb)
        super().__init__()
        self._list = _list
        self.name_button.set_label(self._list.name)
        self.name_entry.get_buffer().set_text(self._list.name, -1)

        drop_target = Gtk.DropTarget.new(ProjectListTask, Gdk.DragAction.MOVE)
        drop_target.set_preload(True)
        drop_target.connect("drop", self.drop_target_drop_cb)
        drop_target.connect("motion", self.drop_target_motion_cb)
        drop_target.connect("leave", self.drop_target_leave_cb)
        drop_target.connect("enter", self.drop_target_enter_cb)
        self.tasks_box.add_controller(drop_target)

        self.tasks_box.set_sort_func(self.tasks_box_sort)
        self.tasks_box.set_filter_func(self.tasks_box_filter)

        for task in read_tasks(self._list.project, self._list._id, False):
            self.tasks_box.append(ProjectListTask(task))

    # Name
    @Gtk.Template.Callback()
    def name_button_clicked_cb(self, *args):
        self.name_button.set_visible(False)   # Entry visible param binded to this
        self.name_entry.grab_focus_without_selecting()

    @Gtk.Template.Callback()
    def name_entry_activate_cb(self, *args):
        self.name_button.set_visible(True)   # Entry visible param binded to this
        self._list.name = self.name_entry.get_buffer().get_text()
        self.name_button.set_label(self._list.name)
        update_list(self._list)

    # New
    @Gtk.Template.Callback()
    def new_button_clicked_cb(self, *args):
        task = create_task("", self._list.project, self._list._id)
        task_ui = ProjectListTask(task)
        self.tasks_box.prepend(task_ui)
        task_ui.name_button.set_visible(False)
        task_ui.name_entry.grab_focus()

    # Delete
    @Gtk.Template.Callback()
    def delete_button_clicked_cb(self, *args):
        self.options_button.popdown()
        dialog = ProjectListDeleteDialog(self)
        dialog.set_transient_for(self.get_root())
        dialog.present()

    # Done tasks
    @Gtk.Template.Callback()
    def show_done_tasks_toggle_button_toggled_cb(self, *args):
        self.options_button.popdown()
        if not self.contain_done_tasks:
            self.contain_done_tasks = True
            for task in read_tasks(self._list.project, self._list._id, True):
                self.tasks_box.append(ProjectListTask(task))
        else:
            self.tasks_box.invalidate_filter()

    def task_done_cb(self, project_list, action_name, value):
        "Remove or filter task row"
        index = value.unpack()
        # Filter or remove row if show done tasks is False
        # prevent from scroll up after filter or remove row
        upper_row = project_list.tasks_box.get_row_at_index(index - 1)
        row = project_list.tasks_box.get_row_at_index(index)
        if not self.contain_done_tasks:
            if upper_row:
                self.get_root().set_focus(upper_row)
            project_list.tasks_box.remove(row)
        elif not self.show_done_tasks_toggle_button.get_active():
            if upper_row:
                self.get_root().set_focus(upper_row)
            row.changed()

    # Scroll - Related to project lists Layout section
    def set_scroll_controller(self):
        self.scroll_controller = Gtk.EventControllerScroll.new(
            Gtk.EventControllerScrollFlags.VERTICAL
        )   # a little tricky. controller send scroll signal even if shift pressed
        self.scroll_controller.connect("scroll", self.scroll_controller_scroll_cb)
        self.scrolled_window.add_controller(self.scroll_controller)

    def scroll_controller_scroll_cb(self, controller, dx, dy):
        # Horizontal scroll project lists scrolled window if shift pressed
        project_lists = self.get_root().project_lists
        view_port = project_lists.get_first_child()
        if project_lists.shift_modifier:
            adjustment = view_port.props.hadjustment
            step = adjustment.get_step_increment()
            adjustment.set_value(adjustment.get_value() + (step * dy)))

    # Drop
    def drop_target_drop_cb(self, target: Gtk.DropTarget, source_row, x, y):
        # source_row moved by motion signal so it should drop on itself
        self.tasks_box.drag_unhighlight_row()
        task_in_db = read_task(source_row.task._id)
        if task_in_db != source_row.task:
            update_task(source_row.task, move_position=True)
        self.get_root().set_focus(source_row)
        return True

    def drop_target_motion_cb(self, target: Gtk.DropTarget, x, y):
        source_row: ProjectListTask = target.get_value()
        target_row: ProjectListTask = self.tasks_box.get_row_at_y(y)

        # None check
        if not source_row or not target_row:
            return 0

        # Move
        if source_row != target_row:
            # index is reverse of position
            source_i = source_row.get_index()
            target_i = target_row.get_index()
            target_p = target_row.task.position
            if source_i == target_i - 1:
                source_row.task.position -= 1
                target_row.task.position +=1
            elif source_i < target_i:
                for i in range(source_i+1, target_i+1):
                    row = self.tasks_box.get_row_at_index(i)
                    row.task.position += 1
                source_row.task.position = target_p
            elif source_i == target_i + 1:
                source_row.task.position += 1
                target_row.task.position -=1
            elif source_i > target_i:
                for i in range(target_i, source_i):
                    row = self.tasks_box.get_row_at_index(i)
                    row.task.position -= 1
                source_row.task.position = target_p

            # Should use invalidate_sort() insteed of changed() for Refresh hightlight shape
            self.tasks_box.invalidate_sort()
            self.tasks_box.drag_unhighlight_row()
            self.tasks_box.drag_highlight_row(source_row)

        # Scroll when mouse near top nad bottom edges
        scrolled_window_height = self.scrolled_window.get_size(Gtk.Orientation.VERTICAL)
        tasks_box_height = self.tasks_box.get_size(Gtk.Orientation.VERTICAL)

        if tasks_box_height > scrolled_window_height:
            adjustment = self.scrolled_window.props.vadjustment
            step = adjustment.get_step_increment() / 3
            v_pos = adjustment.get_value()
            if y - v_pos > 475:
                adjustment.set_value(v_pos + step)
            elif y - v_pos < 25:
                adjustment.set_value(v_pos - step)

        return Gdk.DragAction.MOVE

    def drop_target_leave_cb(self, target: Gtk.DropTarget):
        source_row: ProjectListTask = target.get_value()
        if source_row:
            source_row.moving_out = True
            self.tasks_box.invalidate_filter()

    def drop_target_enter_cb(self, target: Gtk.DropTarget, x, y):
        source_row: ProjectListTask = target.get_value()
        source_row.moving_out = False

        if source_row.task._list == self._list._id:
            self.tasks_box.invalidate_filter()
        else:
            source_row.task._list = self._list._id
            source_row.task.position = find_new_task_position(source_row.task._list)
            source_row.get_parent().remove(source_row)
            self.tasks_box.prepend(source_row)
            self.tasks_box.drag_highlight_row(source_row)

        return Gdk.DragAction.MOVE

    # tasks_box functions
    def focus_on_task(self, target_task):
        if target_task.done:
            self.show_done_tasks_toggle_button.set_active(True)

        first_row = self.tasks_box.get_row_at_index(0)
        target_task_i = first_row.task.position - target_task.position
        target_task_row = self.tasks_box.get_row_at_index(target_task_i)
        GLib.idle_add(lambda *args: self.get_root().set_focus(target_task_row))

    def tasks_box_sort(self, row1: Gtk.ListBoxRow, row2: Gtk.ListBoxRow) -> int:
        return row2.task.position - row1.task.position

    def tasks_box_filter(self, row: Gtk.ListBoxRow) -> bool:
        if row.task.suspended:
            return False
        if not self.show_done_tasks_toggle_button.get_active():
            return not row.task.done
        return not row.moving_out

    def fetch(self, done_tasks):
        tasks = read_tasks(self._list.project, self._list._id, done_tasks)
        for task in tasks:
            self.tasks_box.append(ProjectListTask(task))

