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

from iplan.views.utility_pane import UtilityPane
from iplan.views.page import Page
from iplan.views.primary_menu import PrimaryMenu
from iplan.views.search_bar import SearchBar
from iplan.database.database import Project

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/window.ui")
class IPlanWindow(Adw.ApplicationWindow):
    __gtype_name__ = 'IPlanWindow'
    flap: Adw.Flap = Gtk.Template.Child()

    def __init__(self, **kwargs):
        super().__init__(**kwargs)

