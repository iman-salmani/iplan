from gi.repository import Gtk, Adw

from iplan.views.sidebar.sidebar import Sidebar
from iplan.views.project.project_header import ProjectHeader
from iplan.views.project.project_lists import ProjectLists


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/window.ui")
class IPlanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IPlanWindow'
    # used by children
    flap: Adw.Flap = Gtk.Template.Child()
    project_lists_layout_button: Gtk.Button = Gtk.Template.Child()
    project_lists: ProjectLists = Gtk.Template.Child()

    def __init__(self, **kwargs):
        super().__init__(**kwargs)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/primary_menu.ui")
class PrimaryMenu(Gtk.MenuButton):
    __gtype_name__ = "PrimaryMenu"

