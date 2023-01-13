# window.py
#
# Copyright 2023 Iman Salmani
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

from gi.repository import Gtk, Adw, Gio, GLib

from iplan.views.page import Page
from iplan.views.projects_menu import ProjectsMenu
from iplan.database.database import Project

class IplanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IplanWindow'
    actions = {}
    project: Project = None

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.set_size_request(512, 512)

        # Root
        root = Gtk.Box()
        root.set_orientation(Gtk.Orientation.VERTICAL)
        self.set_content(root)

        # Global actions
        self.create_action("update_project")
        self.create_action("new_project")
        self.create_action("open_project")
        # callbacks using window project attribute like self.props.root.project
        self.create_action("refresh_project_duration")
        self.create_action("new_todo")
        self.create_action("refresh_todos")
        self.create_action(
            "toggle_completed_tasks",
            param=GLib.VariantType("b"),
            state=GLib.Variant("b", False)
        )

        # Header
        header = Adw.HeaderBar()
        header.set_show_start_title_buttons(True)
        root.append(header)

        projects_menu = ProjectsMenu()
        header.pack_start(projects_menu)

        primary_menu = Gtk.MenuButton()
        primary_menu.set_icon_name("open-menu-symbolic")
        header.pack_end(primary_menu)

        menu = Gtk.Popover()
        primary_menu.set_popover(menu)

        # Page
        page = Page()
        root.append(page)

    def create_action(self, name, param:GLib.VariantType=None, state=None, shortcuts=None):
        """Add an window action.

        Args:
            name: the name of the action
            param: parameter
            state:
                if not none create stateful action with
                default state value equal to state argument.
                param required for state
            shortcuts: an optional list of accelerators
            note -> callback add by children
        """

        if state == None:
            action = Gio.SimpleAction.new(name, param)
        else:
            # TODO: check param required
            action = Gio.SimpleAction.new_stateful(name, param, state)

        self.add_action(action)

        if shortcuts:
            self.set_accels_for_action(f"app.{name}", shortcuts)

        self.actions[name] = action
