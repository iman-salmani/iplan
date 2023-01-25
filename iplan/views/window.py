from gi.repository import Gtk, Adw

from iplan.views.sidebar.sidebar import Sidebar
from iplan.views.project.project_header import ProjectHeader
from iplan.views.project.project_lists import ProjectLists


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/window.ui")
class IPlanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IPlanWindow'
    flap: Adw.Flap = Gtk.Template.Child()    # used by children

    def __init__(self, **kwargs):
        super().__init__(**kwargs)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/primary_menu.ui")
class PrimaryMenu(Gtk.MenuButton):
    __gtype_name__ = "PrimaryMenu"

