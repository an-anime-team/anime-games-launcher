use std::collections::VecDeque;

use adw::prelude::*;

use relm4::prelude::*;
use relm4::abstractions::DrawHandler;

const OFFSET: f64 = 10.0;

#[derive(Debug, Clone, Copy)]
pub struct GraphInit {
    /// Width of the DrawingArea.
    pub width: i32,

    /// Height of the DrawingArea.
    pub height: i32,

    /// Amount of last points to display on the graph.
    pub window_size: usize,

    /// Plot lines RGB color from 0.0 to 1.0.
    pub color: (f64, f64, f64)
}

#[derive(Debug)]
pub struct Graph {
    width: i32,
    height: i32,
    window_size: usize,
    color: (f64, f64, f64),

    points: VecDeque<u64>,

    max_point: u64,
    mean_point: u64,

    handler: DrawHandler
}

impl Graph {
    /// Draw stored graph points on the plot.
    fn draw(&mut self) -> Result<(), gtk::cairo::Error> {
        let context = self.handler.get_context();

        let height = self.height as f64;
        let width = self.width as f64;

        let (red, green, blue) = self.color;

        // Clear the plot.
        context.set_operator(gtk::cairo::Operator::Clear);
        context.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        context.paint()?;

        // Configure plot lines.
        context.set_operator(gtk::cairo::Operator::Add);
        context.set_line_width(2.0);

        // Calculate plot scale.
        let x_scale = (width - 2.0 * OFFSET) / (self.window_size as f64 + 1.0);
        let y_scale = (height - 3.0 * OFFSET) / self.max_point as f64;

        // Draw the mean line
        context.set_source_rgba(1.0, 1.0, 1.0, 0.15);
        context.set_dash(&[4.0, 4.0], 0.0);

        // TODO: draw Bézier splines here.
        // context.curve_...

        context.move_to(OFFSET, height - (OFFSET + y_scale * self.mean_point as f64));
        context.line_to(width - OFFSET, height - (OFFSET + y_scale * self.mean_point as f64));

        context.stroke()?;

        // Return solit line style.
        context.set_dash(&[], 0.0);

        // Draw plot points.
        context.move_to(width - OFFSET, height - OFFSET);

        for (i, point) in self.points.iter().enumerate() {
            let x = width - (OFFSET + x_scale * (i + 1) as f64);
            let y = height - OFFSET - *point as f64 * y_scale;

            context.line_to(x, y);
        }

        context.line_to(OFFSET, height - OFFSET);

        context.set_source_rgba(red, green, blue, 0.2);
        context.fill_preserve()?;

        context.set_source_rgba(red, green, blue, 1.0);
        context.stroke()?;

        // Enable antialiasing.
        context.set_antialias(gtk::cairo::Antialias::Good);

        Ok(())
    }
}

#[derive(Debug)]
pub enum GraphMsg {
    SetColor((f64, f64, f64)),
    AddPoint(u64),
    Clear
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for Graph {
    type Init = GraphInit;
    type Input = GraphMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "card",

            #[local_ref]
            area -> gtk::DrawingArea {
                set_content_width: model.width,
                set_content_height: model.height
            }
        }
    }

    async fn init(init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Graph {
            width: init.width,
            height: init.height,
            window_size: init.window_size,
            color: init.color,

            max_point: 0,
            mean_point: 0,

            points: VecDeque::from_iter(vec![0; init.window_size]),
            handler: DrawHandler::new()
        };

        let area = model.handler.drawing_area();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GraphMsg::SetColor(color) => self.color = color,

            GraphMsg::AddPoint(point) => {
                self.points.pop_back();
                self.points.push_front(point);

                // FIXME: it's possible to optimize very well.
                self.max_point = self.points.iter()
                    .copied()
                    .max()
                    .unwrap_or_default();

                self.mean_point = self.points.iter().copied().sum::<u64>() / self.window_size as u64;
            }

            GraphMsg::Clear => {
                self.points = VecDeque::from_iter(vec![0; self.window_size]);

                self.max_point = 0;
                self.mean_point = 0;
            }
        }

        if let Err(err) = self.draw() {
            tracing::error!(?err, "Failed to draw graph");
        }
    }
}
