import sys
import gi
gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')
from gi.repository import Gtk, Gio, GLib, Adw

from iplan.db.manager import check_database
from iplan.db.models.project import Project
from iplan.views.window import IPlanWindow
from iplan.views.search.search_window import SearchWindow

class IPlanApplication(Adw.Application):
    """The main application singleton class."""
    actions = {}
    project: Project = None  # Active project accessible by all windows

    def __init__(self):
        super().__init__(application_id='ir.imansalmani.iplan',
                         flags=Gio.ApplicationFlags.FLAGS_NONE)

        check_database()

        self.create_action(
            'quit',
            callback=lambda *args: self.quit(),
            shortcuts=['<Ctrl>q']
        )
        self.create_action('about', callback=self.on_about)
        self.create_action(
            'shortcuts',
            callback=self.on_shortcuts,
            shortcuts=['<Ctrl>question']
        )
        self.create_action(
            'preferences',
            callback=self.on_preferences,
            shortcuts=['<Ctrl>comma']
        )
        self.create_action("search", callback=self.on_search, shortcuts=["<Ctrl>f"])
        self.create_action("close_modal", callback=self.close_modal, shortcuts=["Escape"])
        self.create_action("update_project")
        self.create_action(
            "open_project",
            param=GLib.VariantType('i') # task id for search
        )
        self.create_action("new_list")
        # callbacks using application project attribute like self.props.root.props.application.project
        self.create_action("refresh_project_duration")
        self.create_action("edit_project")
        self.create_action("new_task", shortcuts=["<Ctrl>n"])
        self.create_action("toggle-project-lists-layout")

    def do_activate(self):
        """Called when the application is activated.

        We raise the application's main window, creating it if
        necessary.
        """
        win = self.props.active_window
        if not win:
            win = IPlanWindow(application=self)
        win.present()

    def on_about(self, widget, _):
        """Callback for the app.about action."""
        about = Adw.AboutWindow(transient_for=self.props.active_window,
                                application_name='iplan',
                                application_icon='ir.imansalmani.iplan',
                                developer_name='Iman Salmani',
                                version='0.1.0',
                                developers=['Iman Salmani'],
                                copyright='© 2023 Iman Salmani')
        about.present()

    def on_shortcuts(self, widget, _):
        shortcuts_window = ShortcutsWindow()
        shortcuts_window.set_transient_for(self.props.active_window)
        shortcuts_window.present()

    def on_preferences(self, widget, _):
        """Callback for the app.preferences action."""
        print('app.preferences action activated')

    def on_search(self, widget, _):
        active_window = self.props.active_window
        if type(active_window) == IPlanWindow:
            window = SearchWindow()
            window.set_transient_for(self.props.active_window)
            window.set_application(self)
            window.present()
        elif type(active_window) == SearchWindow:
            active_window.close()

    def close_modal(self, widget, _):
        if type(self.props.active_window) != IPlanWindow:
            self.props.active_window.close()

    def create_action(
            self,
            name,
            callback=None,
            param: GLib.VariantType=None,
            state=None,
            shortcuts: list[str]=None,
            prefix="app"):
        """Add an window action.

        Args:
            name: the name of the action
            callback: the function to be called when the action is
              activated
            param: parameter
            state:
                if not none create stateful action with
                default state value equal to state argument.
                param required for state
            shortcuts: an optional list of accelerators
            note -> callback can add by children with application.acitons custom property
        """

        if state == None:
            action = Gio.SimpleAction.new(name, param)
        else:
            action = Gio.SimpleAction.new_stateful(name, param, state)

        if callback:
            action.connect("activate", callback)

        self.add_action(action)

        if shortcuts:
            self.set_accels_for_action(f"{prefix}.{name}", shortcuts)

        self.actions[name] = action


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/shortcuts_window.ui")
class ShortcutsWindow(Gtk.ShortcutsWindow):
    __gtype_name__ = "ShortcutsWindow"


def main(version):
    """The application's entry point."""
    app = IPlanApplication()
    return app.run(sys.argv)

