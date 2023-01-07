import gi

from gi.repository import Gtk, Adw, GLib, Pango, Gdk
from time import sleep
from datetime import datetime
from threading import Thread

from iplan.database.database import TodosData, Todo, ProjectsData
from iplan.views.page_header import PageHeader

# Initialize Database connection
todos_data = TodosData()
projects_data = ProjectsData()


class Page(Gtk.Box):
    __gtype_name__ = "Page"
    show_completed_tasks: bool = False
    project = None
    todos_list: Gtk.ListBox = None

    def __init__(self, signal_controller) -> None:
        super().__init__()
        self.signal_controller = signal_controller
        self.set_vexpand(True)
        self.set_hexpand(True)
        self.set_orientation(Gtk.Orientation.VERTICAL)

        # add Handlers
        self.signal_controller.add_handler("project-open", self.opened_project)
        self.signal_controller.add_handler("todo-new", self.new)
        self.signal_controller.add_handler("todo-checked", self.update_todos)
        self.signal_controller.add_handler(
            "todo-toggle_completed", self.toggle_completed_todos
        )

        # Header
        self.header = PageHeader(signal_controller)
        self.append(self.header)

        # Todos list
        scrolled_parent = Gtk.ScrolledWindow()
        scrolled_parent.set_vexpand(True)
        scrolled_parent.set_hexpand(True)
        self.append(scrolled_parent)

        self.todos_list = Gtk.ListBox()
        self.todos_list.set_valign(Gtk.Align.START)
        self.todos_list.add_css_class("boxed-list")
        self.todos_list.set_selection_mode(Gtk.SelectionMode.NONE)
        self.todos_list.set_margin_top(24)
        self.todos_list.set_margin_bottom(24)
        self.todos_list.set_margin_start(24)
        self.todos_list.set_margin_end(24)
        # self.todos_list.set_activate_on_single_click(True)
        # self.todos_list.connect("row-activated", lambda column, row: row.open_todo())
        scrolled_parent.set_child(self.todos_list)

        drop_target = Gtk.DropTarget()
        drop_target.connect("drop", lambda *args: print("drop", *args))
        drop_target.connect("enter", lambda *args: print("enter", *args))
        drop_target.connect("leave", lambda *args: print("leave", *args))
        drop_target.connect("motion", lambda *args: print("motion", *args))
        self.todos_list.add_controller(drop_target)

        self.signal_controller.emit_signal("project-open", projects_data.first())

    # Communicate with database
    def new(self):
        todo = todos_data.add("", project_id=self.project.id)
        todo_ui = TodoRow(todo, self.signal_controller, new=True)
        self.todos_list.prepend(todo_ui)

    def fetch(self):
        todos = todos_data.all(
            show_completed_tasks=self.show_completed_tasks, project=self.project
        )
        for todo in todos:
            todo_ui = TodoRow(todo, self.signal_controller)
            self.todos_list.append(todo_ui)

    # Signal Handlers

    def opened_project(self, project):
        self.project = project
        self.timer_running = False
        self.clear()
        self.fetch()

    def toggle_completed_todos(self, state):
        self.show_completed_tasks = state
        self.clear()
        self.fetch()

    def update_todos(self):
        self.clear()
        self.fetch()

    # UI Functions
    def clear(self):
        while True:
            row = self.todos_list.get_first_child()
            if row:
                self.todos_list.remove(row)
            else:
                break


class TodoRow(Gtk.ListBoxRow):
    __gtype_name__ = "TodoRow"
    timer_running: bool = None
    signal_controller = None
    todo: Todo

    def __init__(self, todo, signal_controller, new=False):
        super().__init__()
        self.signal_controller = signal_controller
        self.todo = todo

        row_box = Gtk.Box()
        row_box.props.orientation = Gtk.Orientation.HORIZONTAL
        row_box.add_css_class("row")
        row_box.set_margin_top(12)
        row_box.set_margin_bottom(12)
        row_box.set_margin_start(12)
        row_box.set_margin_end(12)
        self.set_child(row_box)

        checkbox = Gtk.CheckButton()
        checkbox.set_valign(Gtk.Align.CENTER)
        checkbox.set_active(self.todo.done)
        checkbox.connect(
            "toggled",
            lambda sender: self.clicked_checkbutton(
                Todo(
                    self.todo.id,
                    self.todo.name,
                    checkbox.get_active(),
                    self.todo.project,
                    self.todo.times,
                )
            ),
        )
        row_box.append(checkbox)

        entry = Gtk.Entry()
        entry.set_margin_start(12)
        entry.set_margin_end(12)
        entry.set_hexpand(True)
        entry.set_visible(new)
        entry.add_css_class("heading")
        row_box.append(entry)

        buffer = Gtk.EntryBuffer()
        buffer.set_text(todo.name, -1)
        buffer.connect(
            "inserted-text", lambda *args: self.inserted_text(buffer.get_text())
        )
        entry.set_buffer(buffer)

        button = Gtk.Button()
        button.set_has_frame(False)
        button.set_margin_start(11)
        button.set_visible(not new)
        button.connect(
            "clicked", lambda sender: self.toggle_todo_entry("button", button, entry)
        )
        entry.connect(
            "activate", lambda sender: self.toggle_todo_entry("entry", button, entry)
        )
        row_box.append(button)

        button_separator = Gtk.Separator()
        button_separator.set_hexpand(True)
        button_separator.add_css_class("spacer")
        row_box.append(button_separator)

        label = Gtk.Label()
        label.set_text(todo.name)
        label.set_ellipsize(Pango.EllipsizeMode.END)
        button.set_child(label)

        timer = Gtk.Button()
        timer.set_valign(Gtk.Align.CENTER)
        timer.add_css_class("flat")
        timer.connect(
            "clicked",
            lambda sender: self.toggle_timer(timer, timer.get_child(), self.todo),
        )
        row_box.append(timer)

        timer_content = Adw.ButtonContent()
        timer_content.set_icon_name("preferences-system-time-symbolic")
        duration = todo.get_duration()
        if duration:
            timer_content.set_label(todo.duration_to_text(duration))
        timer.set_child(timer_content)

        last_time = todo.get_last_time()
        if last_time:
            if not last_time[1]:
                self.toggle_timer(timer, timer.get_child(), self.todo, last_time=True)

        delete = Gtk.Button.new_from_icon_name("app-remove-symbolic")
        delete.set_valign(Gtk.Align.CENTER)
        delete.add_css_class("flat")
        delete.connect("clicked", lambda sender: self.delete(self.todo.id, self))
        row_box.append(delete)

        # drag_source = Gtk.DragSource()
        # drag_source.connect("prepare", self.prepare_drag)
        # drag_source.connect("drag-begin", self.drag_begin)
        # self.add_controller(drag_source)

    def prepare_drag(self, drag_source, x, y):
        file = self.get_file()
        pixbuf = self.get_pixbuf()
        print("prepare", drag_source, x, y)

        content_provider = Gdk.ContentProvider.new_union()
        return content_provider

    def drag_begin(self, drag_source, drag):
        paintable = Gtk.WidgetPaintable.new(self)
        drag_source.set_icon(paintable, 0, 0)
        print("drag-begin", drag_source, drag)

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

    def clicked_checkbutton(self, todo):
        todos_data.update(todo)
        self.signal_controller.emit_signal("todo-checked")

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
        self.signal_controller.emit_signal("todo-duration-update")

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

    def toggle_todo_entry(self, sender, button, entry):
        if sender == "button":
            button.set_visible(False)
            entry.set_visible(True)
            entry.grab_focus_without_selecting()
        else:
            entry.set_visible(False)
            button.set_visible(True)
            button.get_child().set_text(entry.get_buffer().get_text())


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

