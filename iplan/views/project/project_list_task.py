import gi
from gi.repository import Gtk, Adw, GLib, Gdk
from time import sleep
from datetime import datetime
from threading import Thread

from iplan.db.models.task import Task
from iplan.db.operations.task import update_task, delete_task


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/project/project_list_task.ui')
class ProjectListTask(Gtk.ListBoxRow):
    __gtype_name__ = "ProjectListTask"
    task: Task
    checkbox: Gtk.CheckButton = Gtk.Template.Child()
    name_entry: Gtk.Entry = Gtk.Template.Child()
    name_button: Gtk.Button = Gtk.Template.Child()
    timer: Gtk.Button = Gtk.Template.Child()
    timer_content: Adw.ButtonContent = Gtk.Template.Child()
    timer_continue_previous = False
    moving_out = False  # When drag

    def __init__(self, task):
        super().__init__()
        self.task = task

        self.checkbox.set_active(self.task.done)
        self.name_button.get_child().set_text(self.task.name)
        self.name_button.set_tooltip_text(self.task.name)
        self.name_entry.get_buffer().set_text(self.task.name, -1)
        name_entry_controller = Gtk.EventControllerKey()
        name_entry_controller.connect(
            "key-released",
            self.name_entry_controller_key_released_cb
        )
        self.name_entry.add_controller(name_entry_controller)

        duration = task.get_duration()
        if duration:
            self.timer_content.set_label(task.duration_to_text(duration))

        if task.done:
            self.timer.set_sensitive(False)
        else:
            self.timer.connect("toggled", self.timer_toggle_button_toggled_cb)

            last_time = task.get_last_time()
            if last_time:
                if not last_time[1]:
                    self.timer_continue_previous = True
                    self.timer.set_active(True)

    # Name
    @Gtk.Template.Callback()
    def name_button_clicked_cb(self, *args):
        self.name_button.set_visible(False) # Entry visible param binded to this
        self.name_entry.grab_focus_without_selecting()

    @Gtk.Template.Callback()
    def name_entry_activate_cb(self, *args):
        self.name_button.set_visible(True)  # Entry visible param binded to this
        self.task.name = self.name_entry.get_buffer().get_text()
        self.name_button.get_child().set_text(self.task.name)
        self.name_button.set_tooltip_text(self.task.name)
        update_task(self.task)

    @Gtk.Template.Callback()
    def name_entry_icon_press_cb(self, *args):  # Cancel name editing
        self.name_button.set_visible(not self.name_button.get_visible())
        self.name_entry.get_buffer().set_text(self.task.name, -1)

    def name_entry_controller_key_released_cb(
            self, controller, keyval, keycode, state):
        if keycode == 9:    # Escape
            self.name_entry.emit("icon-press", Gtk.EntryIconPosition.SECONDARY)

    # Delete
    @Gtk.Template.Callback()
    def delete_button_clicked_cb(self, *args):
        toast_name = self.task.name
        if len(toast_name) > 10:
            toast_name = f"{toast_name[0:9]}..."
        toast = Adw.Toast.new(f'"{toast_name}" deleted')
        toast.set_button_label("Undo")
        toast.connect("button-clicked", self.delete_toast_button_clicked_cb)
        toast.connect("dismissed", self.delete_toast_dismissed_cb)
        self.get_root().toast_overlay.add_toast(toast)
        self.task.suspended = True
        update_task(self.task)
        # Prevent from scroll up after suspend row
        upper_task = self.get_parent().get_row_at_index(self.get_index() - 1)
        if upper_task:
            self.get_root().set_focus(upper_task)
        self.changed()

    def delete_toast_button_clicked_cb(self, *args):    # Undo button
        self.task.suspended = False
        update_task(self.task)

        window = self.get_root()
        if window:   # This happens after open another project
            self.changed()
            window.set_focus(self)

    def delete_toast_dismissed_cb(self, *args):
        if not self.task.suspended: # Checking Undo button
            return

        delete_task(self.task)
        tasks_box = self.get_parent()
        # TODO: tasks_box should removed after open another project
        # this should have None check after fixing memory leak
        # Decrease upper tasks position
        for i in range(0, self.get_index()):
            tasks_box.get_row_at_index(i).task.position -= 1
        tasks_box.remove(self)

    # Open
    # def open_task(self):
    #     window = self.get_root()
    #     modal = TaskModal(self.task)
    #     modal.set_transient_for(window)
    #     modal.present()

    # Done
    @Gtk.Template.Callback()
    def done_check_button_toggled_cb(self, sender):
        active = sender.get_active()

        if self.task.done == active:    # This happens in fetch done tasks
            return

        self.task.done = active
        update_task(self.task)

        if active:
            # Stop timer and disconnect handler
            if self.timer.get_active():
                self.timer.set_active(False)
            self.timer.disconnect_by_func(self.timer_toggle_button_toggled_cb)

            # Remove or filter row
            self.activate_action("task.done", GLib.Variant('i', self.get_index()))
        else:
            self.timer.set_sensitive(True)
            self.timer.connect("toggled", self.timer_toggle_button_toggled_cb)

    # Timer
    def timer_toggle_button_toggled_cb(self, *args):
        if self.timer.get_active():
            self.timer.add_css_class("destructive-action")
            thread = Thread(target=self.start_timer)
            thread.daemon = True
            thread.start()
        else:
            self.timer.remove_css_class("destructive-action")

    def start_timer(self):
        diffrence = None
        if self.timer_continue_previous:
            self.timer_continue_previous = False
            lt = self.task.get_last_time()
            start = datetime.fromtimestamp(lt[0])
            now = datetime.now()
            diffrence = now - start

        else:
            start = datetime.now()
            self.task.duration += f"{start.timestamp()},0;"
            update_task(self.task)

        while self.timer.get_active():
            now = datetime.now()
            diffrence = now - start
            GLib.idle_add(
                lambda: self.timer_content.set_label(self.task.duration_to_text(diffrence.seconds))
            )
            sleep(0.1)

        self.task.duration = self.task.duration[0:-2] + str(diffrence.seconds) + ";"
        self.timer_content.set_label(self.task.get_duration_text())
        update_task(self.task)
        # TODO: change this
        self.activate_action("app.refresh_project_duration")

    # Drag
    @Gtk.Template.Callback()
    def drag_prepare_cb(
            self,
            drag_source: Gtk.DragSource,
            x: float,
            y: float) -> Gdk.ContentProvider:
        if not self.name_entry.get_visible():
            return Gdk.ContentProvider.new_for_value(self)

    @Gtk.Template.Callback()
    def drag_begin_cb(
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
    def drag_cancel_cb(
            self,
            drag_source: Gtk.DragSource,
            drag: Gdk.Drag,
            reason):
        self.moving_out = False
        self.changed()
        return False


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

