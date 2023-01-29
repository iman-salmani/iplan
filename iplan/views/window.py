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
    sidebar: Sidebar = Gtk.Template.Child()
    project_header: ProjectHeader = Gtk.Template.Child()
    project_lists: ProjectLists = Gtk.Template.Child()
    toast_overlay = Gtk.Template.Child()
    settings = None

    def __init__(self, **kwargs):
        self.install_action("list.new", None, lambda *args: self.project_lists.list_new_cb())
        self.install_action("project.open", 'i', self.project_open_cb)

        super().__init__(**kwargs)

        self.settings = Gio.Settings(schema_id="ir.imansalmani.iplan.State")
        if self.settings.get_value("list-layout").unpack() == 1:
            self.layout_button.set_icon_name("view-columns-symbolic")

    def project_open_cb(self, window, action_name, value):
        target_task_id = value.unpack()
        self.project_header.open_project()
        self.project_lists.open_project(target_task_id)
        self.sidebar.projects_section.select_active_project()

    @Gtk.Template.Callback()
    def layout_button_clicked_cb(self, *args):
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

