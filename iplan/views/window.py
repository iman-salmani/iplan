from gi.repository import Gtk, Adw, Gio, GLib

from iplan.db.operations.project import read_projects
from iplan.views.sidebar.sidebar import Sidebar
from iplan.views.project.project_header import ProjectHeader
from iplan.views.project.project_lists import ProjectLists
from iplan.views.project.project_edit_window import ProjectEditWindow

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
        # Install actions
        # Actions should installed before super().__init__
        self.install_action("project.open", None, self.project_open_cb)
        self.install_action("project.edit", None, self.project_edit_cb)
        self.install_action("project.update", None, self.project_update_cb)
        self.install_action(
            "project.delete",
            'i',
            lambda *args: self.sidebar.projects_section.project_delete_cb(*args)
        )
        self.install_action(
            "list.new",
            None,
            lambda *args: self.project_lists.list_new_cb()
        )
        self.install_action(
            "search.task-activate",
            'i',
            self.search_task_activate_cb
        )

        super().__init__(**kwargs)

        # Open first project
        projects = read_projects()
        if not projects:
           self.get_application().project = list(read_projects(archive=True))[0]
        self.get_application().project = list(projects)[0]
        self.activate_action("project.open")

        # Setttings
        # TODO: move this up when set layout per project implemented
        self.settings = Gio.Settings(schema_id="ir.imansalmani.iplan.State")
        if self.settings.get_value("list-layout").unpack() == 1:
            self.layout_button.set_icon_name("view-columns-symbolic")
            self.project_lists.set_layout("horizontal")

    def project_open_cb(self, *args):
        self.project_header.open_project()
        self.project_lists.open_project()

    def project_edit_cb(self, *args):
        window = ProjectEditWindow(self.get_application())
        window.set_transient_for(self)
        window.present()

    def project_update_cb(self, *args):
        self.project_header.open_project()
        self.sidebar.projects_section.refresh()

    def search_task_activate_cb(self, window, action_name, value):
        task_id = value.unpack()
        self.project_header.open_project()
        self.project_lists.open_project(task_id)
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

