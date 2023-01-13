import gi

from gi.repository import Gtk, Adw, GLib, Pango, Gdk
from time import sleep
from datetime import datetime
from threading import Thread

from iplan.database.database import TodosData, Todo, ProjectsData
from iplan.views.page_header import PageHeader
from iplan.views.page_item import TodoRow

# Initialize Database connection
todos_data = TodosData()
projects_data = ProjectsData()

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/page.ui')
class Page(Gtk.Box):
    __gtype_name__ = "Page"
    show_completed_tasks: bool = False
    todos_list: Gtk.ListBox = Gtk.Template.Child()

    def __init__(self) -> None:
        super().__init__()

        # Header
        self.header = PageHeader()
        self.prepend(self.header)

        ## TODO: add drag drop
        #drop_target = Gtk.DropTarget()
        #drop_target.connect("drop", lambda *args: print("drop", *args))
        #drop_target.connect("enter", lambda *args: print("enter", *args))
        #drop_target.connect("leave", lambda *args: print("leave", *args))
        #drop_target.connect("motion", lambda *args: print("motion", *args))
        #self.todos_list.add_controller(drop_target)

        self.connect("map", lambda *args: self.install_actions())


    # Actions
    def install_actions(self):
        actions = self.props.root.actions

        actions["open_project"].connect(
            "activate",
            lambda *args: self.open_project()
        )

        actions["new_todo"].connect("activate", lambda *args: self.new())
        actions["refresh_todos"].connect("activate", lambda *args: self.refresh_todos())

        actions["toggle_completed_tasks"].connect(
            "change-state",
            lambda *args: self.toggle_completed_tasks(bool(args[1]))
        )

        # open first project
        self.props.root.project = projects_data.first()
        self.activate_action("win.open_project")

    def new(self):
        todo = todos_data.add("", project_id=self.props.root.project.id)
        todo_ui = TodoRow(todo, new=True)
        self.todos_list.prepend(todo_ui)

    def open_project(self):
        self.timer_running = False
        self.clear()
        self.fetch()

    def toggle_completed_tasks(self, state):
        self.show_completed_tasks = state
        self.clear()
        self.fetch()

    def refresh_todos(self):
        self.clear()
        self.fetch()

    # UI
    def fetch(self):
        todos = todos_data.all(
            show_completed_tasks=self.show_completed_tasks,
            project=self.props.root.project
        )
        for todo in todos:
            todo_ui = TodoRow(todo)
            self.todos_list.append(todo_ui)

    def clear(self):
        while True:
            row = self.todos_list.get_first_child()
            if row:
                self.todos_list.remove(row)
            else:
                break

