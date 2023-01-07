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

from gi.repository import Gtk, Adw

from iplan.views.page import Page
from iplan.views.projects_menu import ProjectsMenu
from iplan.signal_controller import SignalController

signal_controller = SignalController()

# @Gtk.Template(resource_path='/ir/imansalmani/iplan/window.ui')
class IplanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IplanWindow'

    # label = Gtk.Template.Child()

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.set_size_request(512, 512)

        # Root
        root = Gtk.Box()
        root.set_orientation(Gtk.Orientation.VERTICAL)
        self.set_content(root)

        # Header
        header = Adw.HeaderBar()
        header.set_show_start_title_buttons(True)
        root.append(header)

        projects_menu = ProjectsMenu(signal_controller)
        header.pack_start(projects_menu)

        menu_button = Gtk.MenuButton()
        menu_button.set_icon_name("open-menu-symbolic")
        header.pack_end(menu_button)

        menu = Gtk.Popover()
        menu_button.set_popover(menu)

        # Page
        page = Page(signal_controller)
        root.append(page)
