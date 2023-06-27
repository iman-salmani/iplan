use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::views::snippets::ChartBar;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/snippets/chart.ui")]
    pub struct Chart {
        #[template_child]
        pub overlay: TemplateChild<gtk::Overlay>,
        #[template_child]
        pub levels: TemplateChild<gtk::Box>,
        #[template_child]
        pub lines: TemplateChild<gtk::Box>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Chart {
        const NAME: &'static str = "Chart";
        type Type = super::Chart;
        type ParentType = gtk::Grid;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Chart {}
    impl WidgetImpl for Chart {}
    impl GridImpl for Chart {}
}

glib::wrapper! {
    pub struct Chart(ObjectSubclass<imp::Chart>)
        @extends gtk::Widget, gtk::Grid,
        @implements gtk::Buildable, gtk::Accessible, gtk::ConstraintTarget;
}

impl Default for Chart {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}

#[gtk::template_callbacks]
impl Chart {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_data(
        &self,
        x_labels: Vec<String>,
        percentages: Vec<f64>,
        y_labels: Vec<String>,
        tooltips: Vec<String>,
    ) {
        let imp = self.imp();
        let row: i32 = x_labels.len() as i32;
        let column = y_labels.len() as i32;

        for (i, label) in x_labels.iter().enumerate() {
            self.add_x_label(label, i as i32);

            let bar = ChartBar::new(
                percentages.get(i).unwrap().to_owned(),
                tooltips.get(i).unwrap(),
            );
            imp.levels.append(&bar);
        }

        for (i, label) in y_labels.iter().enumerate() {
            self.add_line();
            self.add_y_label(label, i as i32 + 1);
        }

        let overlay_layout_child = self
            .layout_manager()
            .unwrap()
            .layout_child(&imp.overlay.get());

        overlay_layout_child.set_property("column-span", column);
        overlay_layout_child.set_property("column", 1);
        overlay_layout_child.set_property("row-span", row);
    }

    pub fn clear(&self) {
        let imp = self.imp();

        let bars = imp.levels.observe_children();
        for _ in 0..bars.n_items() {
            let bar = bars.item(0).and_downcast::<gtk::Widget>().unwrap();
            imp.levels.remove(&bar);
        }

        let lines = imp.lines.observe_children();
        for _ in 0..lines.n_items() {
            let bar = lines.item(0).and_downcast::<gtk::Widget>().unwrap();
            imp.lines.remove(&bar);
        }

        self.remove_row(7);
        self.remove_column(0);
    }

    fn add_x_label(&self, label: &str, row: i32) {
        let label = gtk::Label::new(Some(label));
        label.set_vexpand(true);
        label.set_valign(gtk::Align::Center);
        label.add_css_class("dim-label");
        self.attach(&label, 0, row, 1, 1);
    }

    fn add_y_label(&self, label: &str, column: i32) {
        let label = gtk::Label::new(Some(label));
        label.set_hexpand(true);
        label.set_halign(gtk::Align::Start);
        label.add_css_class("dim-label");
        label.add_css_class("chart-y-label");
        self.attach(&label, column, 7, 1, 1);
    }

    fn add_line(&self) {
        let imp = self.imp();
        let separator = gtk::Separator::new(gtk::Orientation::Vertical);
        separator.set_hexpand(true);
        separator.set_vexpand(true);
        separator.set_margin_top(0);
        separator.set_margin_bottom(0);
        separator.set_halign(gtk::Align::Start);
        imp.lines.append(&separator);
    }
}
