import gi
from gi.repository import Gtk, Adw


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search/search_result.ui')
class SearchResult(Gtk.ListBoxRow):
    __gtype_name__ = "SearchResult"
    name = Gtk.Template.Child()
    type_label = Gtk.Template.Child()
    _type = None
    project = None
    task = None

    def __init__(self, _type, name, project=None, task=None):
        super().__init__()
        self._type = _type
        self.project = project
        self.task = task

        self.name.set_label(name)
        self.type_label.set_label(self._type)

