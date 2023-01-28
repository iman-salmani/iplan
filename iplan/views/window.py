from gi.repository import Gtk, Adw, Gio, GLib

from iplan.views.sidebar.sidebar import Sidebar
from iplan.views.project.project_header import ProjectHeader
from iplan.views.project.project_lists import ProjectLists


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/window.ui")
class IPlanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IPlanWindow'
    # used by children
    flap: Adw.Flap = Gtk.Template.Child()
    layout_button: Gtk.Button = Gtk.Template.Child()
    project_lists: ProjectLists = Gtk.Template.Child()
    toast_overlay = Gtk.Template.Child()
    settings = None

    def __init__(self, **kwargs):
        super().__init__(**kwargs)

        self.settings = Gio.Settings(schema_id="ir.imansalmani.iplan.State")
        if self.settings.get_value("list-layout").unpack() == 1:
            self.layout_button.set_icon_name("view-columns-symbolic")
            self.project_lists.set_layout("horizontal")

    @Gtk.Template.Callback()
    def layout_button_clicked(self, *args):
        if self.layout_button.get_icon_name() == "list-symbolic":
            self.layout_button.set_icon_name("view-columns-symbolic")
            self.project_lists.set_layout("horizontal")
            self.settings.set_value("list-layout", GLib.Variant('i', 1))
        else:
            self.layout_button.set_icon_name("list-symbolic")
            self.project_lists.set_layout("vertical")
            self.settings.set_value("list-layout", GLib.Variant('i', 0))


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/primary_menu.ui")
class PrimaryMenu(Gtk.MenuButton):
    __gtype_name__ = "PrimaryMenu"

