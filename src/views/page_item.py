import gi

from gi.repository import Gtk, Adw, GLib, Pango, Gdk
from time import sleep
from datetime import datetime
from threading import Thread

from iplan.database.database import TodosData, Todo

todos_data = TodosData()


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/page_item.ui')
class TodoRow(Gtk.ListBoxRow):
    __gtype_name__ = "TodoRow"
    timer_running: bool = None
    check_button: Gtk.CheckButton = Gtk.Template.Child()
    name_entry: Gtk.Entry = Gtk.Template.Child()
    name_entry_buffer: Gtk.EntryBuffer = Gtk.Template.Child()
    name_button: Gtk.Button = Gtk.Template.Child()
    name_button_label: Gtk.Label = Gtk.Template.Child()
    timer: Gtk.Button = Gtk.Template.Child()
    timer_content: Adw.ButtonContent = Gtk.Template.Child()
    delete_button: Gtk.Button = Gtk.Template.Child()
    todo: Todo

    def __init__(self, todo, new=False):
        super().__init__()
        self.todo = todo

        self.check_button.set_active(self.todo.done)
        self.check_button.connect(
            "toggled",
            lambda sender: self.clicked_check_button(
                Todo(
                    self.todo.id,
                    self.todo.name,
                    self.check_button.get_active(),
                    self.todo.project,
                    self.todo.times,
                )
            ),
        )

        self.name_entry.set_visible(new)
        self.name_entry_buffer.set_text(todo.name, -1)
        self.name_entry_buffer.connect(
            "inserted-text", lambda *args: self.inserted_text(self.name_entry_buffer.get_text())
        )
        self.name_entry.connect(
            "activate",
            lambda sender: self.toggle_todo_entry("entry")
        )

        self.name_button.set_visible(not new)
        self.name_button.connect(
            "clicked",
            lambda sender: self.toggle_todo_entry("button")
        )
        self.name_button_label.set_text(todo.name)

        self.timer.connect(
            "clicked",
            lambda sender: self.toggle_timer(self.timer, self.timer.get_child(), self.todo),
        )
        duration = todo.get_duration()
        if duration:
            self.timer_content.set_label(todo.duration_to_text(duration))

        last_time = todo.get_last_time()
        if last_time:
            if not last_time[1]:
                self.toggle_timer(self.timer, self.timer_content, self.todo, last_time=True)

        self.delete_button.connect("clicked", lambda sender: self.delete(self.todo.id, self))

    #def prepare_drag(self, drag_source, x, y):
    #    file = self.get_file()
    #    pixbuf = self.get_pixbuf()
    #    print("prepare", drag_source, x, y)
    #
    #    content_provider = Gdk.ContentProvider.new_union()
    #    return content_provider

    #def drag_begin(self, drag_source, drag):
    #    paintable = Gtk.WidgetPaintable.new(self)
    #    drag_source.set_icon(paintable, 0, 0)
    #    print("drag-begin", drag_source, drag)

    # Actions
    def toggle_todo_entry(self, sender):
        if sender == "button":
            self.name_button.set_visible(False)
            self.name_entry.set_visible(True)
            self.name_entry.grab_focus_without_selecting()
        else:
            self.name_entry.set_visible(False)
            self.name_button.set_visible(True)
            self.name_button.get_child().set_text(self.name_entry_buffer.get_text())

    def delete(self, _id, todoWidget):
        todos_data.delete(_id)
        self.get_parent().remove(todoWidget)

    def open_todo(self):
        window = self.get_root()
        modal = TodoModal(self.todo)
        modal.set_transient_for(window)
        modal.present()

    def inserted_text(self, text):
        self.todo.name = text
        todos_data.update(self.todo)

    def clicked_check_button(self, todo):
        todos_data.update(todo)
        self.activate_action("win.refresh_todos")

    def toggle_timer(self, button, content, todo, last_time=False):
        if button.has_css_class("flat"):
            button.remove_css_class("flat")
            button.add_css_class("destructive-action")

            self.timer_running = True
            thread = Thread(target=self.start_timer, args=(content, todo, last_time))
            thread.daemon = True
            thread.start()
        else:
            self.timer_running = False

            button.add_css_class("flat")
            button.remove_css_class("destructive-action")

    # UI
    def start_timer(self, content, todo: Todo, last_time):
        todos_data = TodosData()  # for new thread
        diffrence = None
        if last_time:
            lt = todo.get_last_time()
            start = datetime.fromtimestamp(lt[0])
            now = datetime.now()
            diffrence = now - start

        else:
            start = datetime.now()
            todo.times = todo.times + f"{start.timestamp()},0;"
            todos_data.update(todo)

        while self.timer_running:
            now = datetime.now()
            diffrence = now - start
            text = ""
            GLib.idle_add(
                lambda: content.set_label(todo.duration_to_text(diffrence.seconds))
            )
            sleep(0.1)

        todo.times = todo.times[0:-2] + str(diffrence.seconds) + ";"
        content.set_label(todo.get_duration_text())
        todos_data.update(todo)
        self.activate_action("win.refresh_project_duration")


class TodoModal(Adw.Window):
    __gtype_name__ = "TodoModal"

    def __init__(self, todo: Todo):
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

        title = Gtk.Label.new(todo.name)
        title.add_css_class("title-1")
        content.append(title)

