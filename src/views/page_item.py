import gi

from gi.repository import Gtk, Adw, GLib, Pango, Gdk
from time import sleep
from datetime import datetime
from threading import Thread

from iplan.database.database import TasksData, Task

tasks_data = TasksData()


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/page_item.ui')
class TaskRow(Gtk.ListBoxRow):
    __gtype_name__ = "TaskRow"
    timer_running: bool = None
    checkbox: Gtk.CheckButton = Gtk.Template.Child()
    name_entry: Gtk.Entry = Gtk.Template.Child()
    name_entry_buffer: Gtk.EntryBuffer = Gtk.Template.Child()
    name_button: Gtk.Button = Gtk.Template.Child()
    name_button_label: Gtk.Label = Gtk.Template.Child()
    timer: Gtk.Button = Gtk.Template.Child()
    timer_content: Adw.ButtonContent = Gtk.Template.Child()
    delete_button: Gtk.Button = Gtk.Template.Child()
    task: Task

    def __init__(self, task, new=False):
        super().__init__()
        self.task = task

        self.checkbox.set_active(self.task.done)

        self.name_entry.set_visible(new)
        self.name_entry_buffer.set_text(task.name, -1)
        self.name_entry.connect(
            "activate",
            lambda sender: self.toggle_task_entry("entry")
        )

        self.name_button.set_visible(not new)
        self.name_button.connect(
            "clicked",
            lambda sender: self.toggle_task_entry("button")
        )
        self.name_button_label.set_text(task.name)

        duration = task.get_duration()
        if duration:
            self.timer_content.set_label(task.duration_to_text(duration))

        if task.done:
            self.timer.set_sensitive(False)
        else:
            self.timer.connect(
                "clicked",
                lambda sender: self.toggle_timer(),
            )

            last_time = task.get_last_time()
            if last_time:
                if not last_time[1]:
                    self.toggle_timer(last_time=True)

    # Actions
    def toggle_task_entry(self, sender):
        if sender == "button":
            self.name_button.set_visible(False)
            self.name_entry.set_visible(True)
            self.name_entry.grab_focus_without_selecting()
        else:
            self.name_entry.set_visible(False)
            self.name_button.set_visible(True)
            self.name_button.get_child().set_text(self.name_entry_buffer.get_text())

    @Gtk.Template.Callback()
    def delete(self, sender):
        tasks_data.delete(self.task.id)
        self.get_parent().remove(self)

    def open_task(self):
        window = self.get_root()
        modal = TaskModal(self.task)
        modal.set_transient_for(window)
        modal.present()

    @Gtk.Template.Callback()
    def change_task_name(self, buffer, cursor, text, length):
        self.task.name = buffer.get_text()
        tasks_data.update(self.task)

    @Gtk.Template.Callback()
    def toggled_checkbox(self, sender):
        self.task.done = sender.get_active()
        tasks_data.update(self.task)
        if self.timer_running:
            self.toggle_timer()
        self.activate_action("app.refresh_tasks")

    def toggle_timer(self, last_time=False):
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
        return Gdk.ContentProvider.new_for_value(self)

    @Gtk.Template.Callback()
    def on_drag_begin(
            self, drag_source: Gtk.DragSource,
            drag: Gdk.Drag) -> None:
        allocation = self.get_allocation()
        drag_widget = Gtk.ListBox()
        drag_widget.set_size_request(allocation.width, allocation.height)

        drag_row = TaskRow(self.task)
        drag_widget.append(drag_row)
        drag_widget.drag_highlight_row(drag_row)

        drag_icon = Gtk.DragIcon.get_for_drag(drag)
        drag_icon.props.child = drag_widget
        drag.set_hotspot(0, 0)

    def start_timer(self, last_time):
        tasks_data = TasksData()  # for new thread
        diffrence = None
        if last_time:
            lt = self.task.get_last_time()
            start = datetime.fromtimestamp(lt[0])
            now = datetime.now()
            diffrence = now - start

        else:
            start = datetime.now()
            self.task.times += f"{start.timestamp()},0;"
            tasks_data.update(self.task)

        while self.timer_running:
            now = datetime.now()
            diffrence = now - start
            text = ""
            GLib.idle_add(
                lambda: self.timer_content.set_label(self.task.duration_to_text(diffrence.seconds))
            )
            sleep(0.1)

        self.task.times = self.task.times[0:-2] + str(diffrence.seconds) + ";"
        self.timer_content.set_label(self.task.get_duration_text())
        tasks_data.update(self.task)
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

