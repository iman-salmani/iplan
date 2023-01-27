import gi
from gi.repository import Gtk, Adw, GLib, Gdk
from time import sleep
from datetime import datetime
from threading import Thread
import copy

from iplan.db.models.task import Task
from iplan.db.operations.task import update_task, delete_task


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/project/project_list_task.ui')
class ProjectListTask(Gtk.ListBoxRow):
    __gtype_name__ = "ProjectListTask"
    timer_running: bool = None
    checkbox: Gtk.CheckButton = Gtk.Template.Child()
    name_entry: Gtk.Entry = Gtk.Template.Child()
    name_button: Gtk.Button = Gtk.Template.Child()
    timer: Gtk.Button = Gtk.Template.Child()
    timer_content: Adw.ButtonContent = Gtk.Template.Child()
    delete_button: Gtk.Button = Gtk.Template.Child()
    task: Task
    moving_out: bool = False    # when drag!

    def __init__(self, task, new=False):
        super().__init__()
        self.task = task

        self.checkbox.set_active(self.task.done)
        self.name_button.set_visible(not new)
        self.name_button.get_child().set_text(self.task.name)
        self.name_entry.get_buffer().set_text(self.task.name, -1)
        entry_controller = Gtk.EventControllerKey()
        entry_controller.connect("key-released", self.on_name_entry_key_released)
        self.name_entry.add_controller(entry_controller)

        duration = task.get_duration()
        if duration:
            self.timer_content.set_label(task.duration_to_text(duration))

        if task.done:
            self.timer.set_sensitive(False)
        else:
            self.timer.connect("clicked", self.toggle_timer)

            last_time = task.get_last_time()
            if last_time:
                if not last_time[1]:
                    self.toggle_timer(last_time=True)

    # Actions
    @Gtk.Template.Callback()
    def on_name_toggled(self, *args):
        # used by both name entry and name button
        # name_entry have binding to name button visibility
        name_button_visible = not self.name_button.get_visible()
        self.name_button.set_visible(name_button_visible)
        if name_button_visible:
            self.task.name = self.name_entry.get_buffer().get_text()
            self.name_button.get_child().set_text(self.task.name)
            update_task(self.task)
        else:
            self.name_entry.grab_focus_without_selecting()

    @Gtk.Template.Callback()
    def on_name_entry_canceled(self, *args):
        self.name_button.set_visible(not self.name_button.get_visible())
        self.name_entry.get_buffer().set_text(self.task.name, -1)

    def on_name_entry_key_released(self, controller, keyval, keycode, state):
        if keycode == 9:    # Escape
            self.on_name_entry_canceled()

    @Gtk.Template.Callback()
    def delete(self, *args):
        delete_task(self.task)
        deleted_task_i = self.get_index()
        tasks_box = self.get_parent()
        # prevent from scroll up after remove row
        upper_task = tasks_box.get_row_at_index(self.get_index() - 1)
        if upper_task:
            self.get_root().set_focus(upper_task)
        tasks_box.remove(self)

        # decrease upper tasks position
        for i in range(0, deleted_task_i):
            tasks_box.get_row_at_index(i).task.position -= 1

    def open_task(self):
        window = self.get_root()
        modal = TaskModal(self.task)
        modal.set_transient_for(window)
        modal.present()

    @Gtk.Template.Callback()
    def toggled_checkbox(self, sender):
        active = sender.get_active()

        if self.task.done == active:
            # this happens in fetch done tasks
            return

        self.task.done = active
        update_task(self.task)

        if active:
            # stop timer and disconnect handler
            if self.timer_running:
                self.toggle_timer()
            self.timer.disconnect_by_func(self.toggle_timer)

            # prevent from scroll up after filter or remove row
            upper_task = self.get_parent().get_row_at_index(self.get_index() - 1)

            # filter or remove row if done tasks filter is not False
            project_list = self.get_parent().get_parent()
            if self.get_root().project_lists_layout_button.get_icon_name() == "view-columns-symbolic":
                # in board view tasks_box have scrolled_window parent
                project_list = project_list.get_parent().get_parent()
            # because when is None ProjectList do not fetched done tasks
            if project_list.filter_done_tasks == None:
                if upper_task:
                    self.get_root().set_focus(upper_task)
                self.get_parent().remove(self)
            elif project_list.filter_done_tasks == True:
                if upper_task:
                    self.get_root().set_focus(upper_task)
                self.changed()
        else:
            self.timer.set_sensitive(True)
            self.timer.connect("clicked", self.toggle_timer)

    def toggle_timer(self, *args, last_time=False):
        if self.timer.has_css_class("flat"):
            self.timer.remove_css_class("flat")
            self.timer.add_css_class("destructive-action")

            self.timer_running = True
            thread = Thread(target=self.start_timer, args=(last_time, ))
            thread.daemon = True
            thread.start()
        else:
            self.timer_running = False

            self.timer.add_css_class("flat")
            self.timer.remove_css_class("destructive-action")

    # UI
    @Gtk.Template.Callback()
    def on_drag_prepare(self, drag_source: Gtk.DragSource,
            x: float, y: float) -> Gdk.ContentProvider:
        if not self.name_entry.get_visible():
            return Gdk.ContentProvider.new_for_value(self)

    @Gtk.Template.Callback()
    def on_drag_begin(
            self, drag_source: Gtk.DragSource,
            drag: Gdk.Drag) -> None:
        #allocation = self.get_allocation()
        #drag_widget = Gtk.ListBox()
        #drag_widget.set_size_request(240, allocation.height)

        #drag_row = ProjectListTask(self.task)
        #drag_row.delete_button.set_visible(False)
        #drag_row.timer.set_visible(False)
        #drag_widget.append(drag_row)
        #drag_widget.drag_highlight_row(drag_row)
        #drag_row.set_size_request(240, 64)

        drag_icon = Gtk.DragIcon.get_for_drag(drag)
        drag_icon.props.child = Gtk.Label()
        drag.set_hotspot(0, 0)

    @Gtk.Template.Callback()
    def on_drag_cancel(
            self,
            drag_source: Gtk.DragSource,
            drag: Gdk.Drag,
            reason):
        # its probably canceled
        self.moving_out = False
        self.get_parent().invalidate_filter()
        return False

    def start_timer(self, last_time):
        diffrence = None
        if last_time:
            lt = self.task.get_last_time()
            start = datetime.fromtimestamp(lt[0])
            now = datetime.now()
            diffrence = now - start

        else:
            start = datetime.now()
            self.task.duration += f"{start.timestamp()},0;"
            update_task(self.task)

        while self.timer_running:
            now = datetime.now()
            diffrence = now - start
            text = ""
            GLib.idle_add(
                lambda: self.timer_content.set_label(self.task.duration_to_text(diffrence.seconds))
            )
            sleep(0.1)

        self.task.duration = self.task.duration[0:-2] + str(diffrence.seconds) + ";"
        self.timer_content.set_label(self.task.get_duration_text())
        update_task(self.task)
        self.activate_action("app.refresh_project_duration")


class TaskModal(Adw.Window):
    __gtype_name__ = "TaskModal"

    def __init__(self, task: Task):
        super().__init__()
        self.set_modal(True)
        self.set_size_request(480, 480)

        content = Gtk.Box()
        content.set_orientation(Gtk.Orientation.VERTICAL)
        self.set_content(content)

        header = Adw.HeaderBar()
        header.add_css_class("flat")
        header.set_title_widget(Gtk.Label())
        content.append(header)

        title = Gtk.Label.new(task.name)
        title.add_css_class("title-1")
        content.append(title)


