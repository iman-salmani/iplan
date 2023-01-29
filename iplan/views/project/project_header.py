import gi
from gi.repository import Gtk, Adw


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_header.ui")
class ProjectHeader(Gtk.Box):
    __gtype_name__ = "ProjectHeader"
    name_label: Gtk.Label = Gtk.Template.Child()
    duration_button_content: Adw.ButtonContent = Gtk.Template.Child()
    stat_box: Gtk.Box = Gtk.Template.Child()

    def __init__(self):
        super().__init__()

    # Open - used by project_open_cb and project_update_cb in window
    def open_project(self):
        self.name_label.set_text(self.props.root.props.application.project.name)
        self.refresh_project_duration()

    def refresh_project_duration(self, *args):
        duration = self.props.root.props.application.project.get_duration()
        if duration:
            self.duration_button_content.set_label(
                self.props.root.props.application.project.duration_to_text(duration)
            )
        else:
            self.duration_button_content.set_label("")

        table = self.props.root.props.application.project.get_duration_table()

        while True:
            row = self.stat_box.get_first_child()
            if row:
                self.stat_box.remove(row)
            else:
                break

        dates = list(table.keys())
        dates.sort()
        dates.reverse()
        for date in dates:
            box = Gtk.Box()
            box.set_margin_top(9)
            box.set_margin_bottom(9)
            box.set_margin_start(9)
            box.set_margin_end(9)
            self.stat_box.append(box)

            date_label = Gtk.Label()
            date_label.set_text(date.strftime("%d %b"))
            date_label.set_margin_end(18)
            date_label.set_hexpand(True)
            date_label.set_halign(Gtk.Align.START)
            box.append(date_label)

            duration_label = Gtk.Label()
            duration_label.set_text(self.props.root.props.application.project.duration_to_text(table[date]))
            box.append(duration_label)

            if date != dates[-1]:
                self.stat_box.append(Gtk.Separator())

