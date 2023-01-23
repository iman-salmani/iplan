import gi
from gi.repository import Gtk

from iplan.db.operations.project import search_projects
from iplan.db.operations.task import search_tasks
from iplan.views.search_bar.search_result import SearchResult

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search_bar/search_bar.ui')
class SearchBar(Gtk.Box):
    __gtype_name__ = "SearchBar"
    search_entry = Gtk.Template.Child()
    menu = Gtk.Template.Child()
    founded_items = Gtk.Template.Child()

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.search_entry.set_key_capture_widget(self.menu)
        self.connect("map", self.on_mapped)

    # Actions
    def on_mapped(self, *args):
        self.disconnect_by_func(self.on_mapped)
        self.props.root.props.application.actions["search"].connect(
            "activate",
            lambda *args: self.search_entry.grab_focus()
        )

    @Gtk.Template.Callback()
    def on_search_entry_changed(self, sender):
        self.clear()

        text = sender.get_text().lower()

        for project in search_projects(text):
            self.founded_items.append(
                SearchResult("project", project.name, self.menu, project=project))
        for task in search_tasks(text):
            self.founded_items.append(
                SearchResult("task", task.name, self.menu, task=task))

        first_item = self.founded_items.get_first_child()
        if first_item:
            self.menu.popup()
            first_item.grab_focus() # because of search entry have key capture
        else:
            self.menu.popdown() # auto hide property getting focus of search entry
            self.search_entry.grab_focus()

    # UI
    def clear(self):
        while True:
            row = self.founded_items.get_first_child()
            if row:
                self.founded_items.remove(row)
            else:
                break

