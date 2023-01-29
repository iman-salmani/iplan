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
    show_done_tasks_toggle_button = Gtk.Template.Child()

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.set_focus(self.search_entry)
        search_entry_controller = Gtk.EventControllerKey()
        search_entry_controller.connect("key-pressed", self.search_entry_controller_key_pressed_cb)
        self.search_entry.add_controller(search_entry_controller)

    @Gtk.Template.Callback()
    def search_entry_activate_cb(self, *args):
        selected_row = self.search_results.get_selected_row()
        first_row = self.search_results.get_first_child()

        if type(first_row) != SearchResult: # Check placeholder
            return

        if selected_row:
            self.search_results_row_activated_cb(self.search_results, selected_row)
        elif first_row:
            self.search_results_row_activated_cb(self.search_results, first_row)

    def search_entry_controller_key_pressed_cb(self, controller, keyval, keycode, state):
        key = Gdk.keyval_name(keyval)
        first_child = self.search_results.get_first_child()

        if type(first_child) != SearchResult:   # Check placeholder
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
    def search_results_row_activated_cb(self, list_box, row):
        # Also used by search_entry_activate_cb
        if row._type == "project":
            self.get_application().project = row.project
            self.get_toplevels()[0].activate_action("project.open")
        elif row._type == "task":
            self.get_application().project = \
                read_project(row.task.project)
            self.get_toplevels()[0].activate_action(
                "search.task-activate",
                GLib.Variant("i", row.task._id)
            )
        self.close()

    @Gtk.Template.Callback()
    def search_entry_search_changed_cb(self, *args):
        # Also used by show_done_tasks_toggle_button toggled signal
        while True:
            row = self.search_results.get_first_child()
            if row:
                self.search_results.remove(row)
            else:
                break

        text = self.search_entry.get_text().lower()
        if not text.strip():
            return

        for project in search_projects(text):
            self.search_results.append(
                SearchResult("project", project.name, project=project))
        for task in search_tasks(text, done=self.show_done_tasks_toggle_button.get_active()):
            self.search_results.append(
                SearchResult("task", task.name, task=task))

        first_item = self.search_results.get_first_child()
        if not first_item:
            self.search_entry.grab_focus()
            self.search_results.set_placeholder(self.search_results_placeholder)

