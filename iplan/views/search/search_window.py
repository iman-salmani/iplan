import gi
from gi.repository import Gtk, Adw, Gdk, GLib

from iplan.db.operations.project import read_project, search_projects
from iplan.db.operations.task import search_tasks
from iplan.views.search.search_result import SearchResult

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search/search_window.ui')
class SearchWindow(Gtk.Window):
    __gtype_name__ = "SearchWindow"
    search_entry = Gtk.Template.Child()
    search_results = Gtk.Template.Child()
    search_results_placeholder = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.set_focus(self.search_entry)
        controller = Gtk.EventControllerKey()
        controller.connect("key-pressed", self.on_key_pressed)
        self.search_entry.add_controller(controller)
        self.search_entry.connect("activate", self.on_search_entry_activated)
        self.search_results.connect("row-activated", self.on_result_activated)

    def on_search_entry_activated(self, *args):
        selected_row = self.search_results.get_selected_row()
        first_row = self.search_results.get_first_child()

        if type(first_row) != SearchResult:
            return

        if selected_row:
            self.on_result_activated(self.search_results, selected_row)
        elif first_row:
            self.on_result_activated(self.search_results, first_row)

    def on_key_pressed(self, controller, keyval, keycode, state):
        key = Gdk.keyval_name(keyval)
        first_child = self.search_results.get_first_child()
        if type(first_child) != SearchResult:
            return

        arrows = [65364, 65362] # Down, Up
        if keyval in arrows:
            move = 1
            if keyval == 65362:
                move = -1
            selected_row = self.search_results.get_selected_row()
            if selected_row:
                self.search_results.select_row(
                    self.search_results.get_row_at_index(
                        selected_row.get_index() + move
                    )
                )
            else:
                self.search_results.select_row(first_child)

    @Gtk.Template.Callback()
    def on_result_activated(self, list_box, row):
        if row._type == "project":
            self.get_application().project = row.project
            self.activate_action("app.open_project", GLib.Variant("i", -1))
        elif row._type == "task":
            self.get_application().project = \
                read_project(row.task.project)
            self.activate_action(
                "app.open_project",
                GLib.Variant("i", row.task._id)
            )
        self.get_application().actions["open-searched"].activate()
        self.close()

    @Gtk.Template.Callback()
    def on_search_entry_changed(self, sender):
        self.clear()

        text = sender.get_text().lower()
        if not text.strip():
            return

        for project in search_projects(text):
            self.search_results.append(
                SearchResult("project", project.name, project=project))
        for task in search_tasks(text):
            self.search_results.append(
                SearchResult("task", task.name, task=task))

        first_item = self.search_results.get_first_child()
        if not first_item:
            self.search_entry.grab_focus()
            self.search_results.set_placeholder(self.search_results_placeholder)

    # UI
    def clear(self):
        while True:
            row = self.search_results.get_first_child()
            if row:
                self.search_results.remove(row)
            else:
                break

