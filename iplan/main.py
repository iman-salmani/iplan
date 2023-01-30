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
        self.create_action('about', callback=self.app_about_cb)
        self.create_action(
            'shortcuts',
            callback=self.app_shortcuts_cb,
            shortcuts=['<Ctrl>question']
        )
        self.create_action(
            'preferences',
            callback=self.app_preferences_cb,
            shortcuts=['<Ctrl>comma']
        )
        self.create_action(
            "search",
            callback=self.app_search_cb,
            shortcuts=["<Ctrl>f"]
        )
        self.create_action(
            "modal-close",
            callback=self.app_modal_close_cb,
            shortcuts=["Escape"]
        )

    def do_activate(self):
        """Called when the application is activated.

        We raise the application's main window, creating it if
        necessary.
        """
        win = self.props.active_window
        if not win:
            win = IPlanWindow(application=self)
        win.present()

    def app_about_cb(self, widget, _):
        """Callback for the app.about action."""
        about = Adw.AboutWindow(transient_for=self.props.active_window,
                                application_name='iplan',
                                application_icon='ir.imansalmani.iplan',
                                developer_name='Iman Salmani',
                                version='0.1.0',
                                developers=['Iman Salmani'],
                                copyright='© 2023 Iman Salmani')
        about.present()

    def app_shortcuts_cb(self, widget, _):
        shortcuts_window = Gtk.Builder.new_from_resource(
            "/ir/imansalmani/iplan/ui/shortcuts_window.ui"
        ).get_object("shortcuts_window")
        shortcuts_window.set_transient_for(self.props.active_window)
        shortcuts_window.present()

    def app_preferences_cb(self, widget, _):
        """Callback for the app.preferences action."""
        print('app.preferences action activated')

    def app_search_cb(self, widget, _):
        active_window = self.props.active_window
        if type(active_window) == IPlanWindow:
            window = SearchWindow(
                application=self,
                transient_for=self.props.active_window
            )
            window.present()
        elif type(active_window) == SearchWindow:
            active_window.close()

    def app_modal_close_cb(self, widget, _):
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


def main(version):
    """The application's entry point."""
    app = IPlanApplication()
    return app.run(sys.argv)

