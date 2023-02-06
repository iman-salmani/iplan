from gi.repository import Gtk

from iplan.views.sidebar.sidebar_projects import SidebarProjects


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar.ui")
class Sidebar(Gtk.Box):
    __gtype_name__ = "Sidebar"
    projects_section: SidebarProjects = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
