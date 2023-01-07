import gi
from gi.repository import Gtk, Adw

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


class ProjectsMenu(Gtk.Box):
    __gtype_name__ = "ProjectsMenu"
    projects_list: Gtk.ListBox = None
    popover: Gtk.Popover = None
    no_results: Gtk.Label = False
    archive: bool = False

    def __init__(self, signal_controller) -> None:
        super().__init__()

        self.set_margin_start(12)
        self.signal_controller = signal_controller

        # add handlers
        self.signal_controller.add_handler("project-update", self.refresh_projects)
        self.signal_controller.add_handler(
            "project-open", lambda project: self.menu_button.set_label(project.name)
        )

        # Menu button
        self.menu_button = Gtk.MenuButton()
        self.menu_button.add_css_class("flat")
        self.menu_button.set_label("Projects")
        self.append(self.menu_button)

        self.popover = Gtk.Popover()
        self.popover.add_css_class("menu")
        self.popover.set_halign(Gtk.Align.START)
        self.popover.set_valign(Gtk.Align.END)
        self.popover.set_size_request(250, 100)
        self.menu_button.set_popover(self.popover)

        column = Gtk.Box()
        column.set_orientation(Gtk.Orientation.VERTICAL)
        self.popover.set_child(column)

        header = Gtk.Box()
        header.set_spacing(6)
        header.set_margin_top(6)
        header.set_margin_start(6)
        header.set_margin_end(6)
        column.append(header)

        search = Gtk.SearchEntry()
        search.connect("search-changed", self.search_changed)
        header.append(search)

        archive_button = Gtk.Button.new_from_icon_name("folder-open-symbolic")
        archive_button.connect("clicked", self.toggle_archive)
        archive_button.set_has_frame(False)
        header.append(archive_button)

        column.append(Gtk.Separator())

        self.no_results = Gtk.Label.new("No Results Found")
        self.no_results.set_visible(False)
        self.no_results.add_css_class("heading")
        self.no_results.set_halign(Gtk.Align.CENTER)
        self.no_results.set_margin_top(18)
        self.no_results.set_margin_bottom(24)
        column.append(self.no_results)

        self.projects_list = Gtk.Box()
        self.projects_list.set_orientation(Gtk.Orientation.VERTICAL)
        self.projects_list.set_margin_bottom(6)
        self.projects_list.set_margin_start(6)
        self.projects_list.set_margin_end(6)
        column.append(self.projects_list)

        self.fetch()

        new_button = Gtk.Button.new_from_icon_name("tab-new-symbolic")
        new_button.set_has_frame(False)
        new_button.connect("clicked", self.new)
        self.append(new_button)

    def new(self, sender):
        name = "New Project"
        project = Project(id=projects_data.add(name), name=name, archive=False)
        self.projects_list.append(self.create_project_ui(project))
        self.signal_controller.emit_signal("project-open", project)

    def clear(self):
        while True:
            row = self.projects_list.get_first_child()
            if row:
                self.projects_list.remove(row)
            else:
                break

    def search_changed(self, sender):
        self.clear()

        text = sender.get_text()
        if text:
            projects = projects_data.search(text, archive=self.archive)
            if projects:
                for project in projects:
                    self.projects_list.append(self.create_project_ui(project))

                self.no_results.set_visible(False)
                self.projects_list.set_visible(True)
            else:
                self.projects_list.set_visible(False)
                self.no_results.set_visible(True)
        else:
            self.fetch()
            self.no_results.set_visible(False)
            self.projects_list.set_visible(True)

    def fetch(self):
        for project in projects_data.all(archive=self.archive):
            self.projects_list.append(self.create_project_ui(project))

    def refresh_projects(self):
        self.clear()
        self.fetch()

    def toggle_archive(self, sender):
        sender.set_has_frame(not sender.get_has_frame())
        self.archive = not self.archive

        self.refresh_projects()

    def open_project(self, project: Project):
        self.signal_controller.emit_signal("project-open", project)
        self.popover.popdown()

    def create_project_ui(self, project: Project):
        button = Gtk.Button()
        button.set_hexpand(True)
        button.add_css_class("flat")

        label = Gtk.Label.new(project.name)
        label.add_css_class("heading")
        label.set_halign(Gtk.Align.START)
        button.set_child(label)

        button.connect(
            "clicked",
            lambda sender: self.open_project(project),
        )

        return button

