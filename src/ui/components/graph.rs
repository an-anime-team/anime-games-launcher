use std::{collections::VecDeque, time::Duration};

use tokio::time::sleep;

use adw::prelude::*;
use gtk::prelude::*;

use gtk::cairo::{Context, Operator};

use relm4::abstractions::DrawHandler;
use relm4::prelude::*;

#[derive(Debug)]
pub struct GraphInit {
    /// width of DrawingArea
    width: i32,
    /// height of DrawingArea
    height: i32,
    /// rgb 0.0 to 1.0
    color: (f64, f64, f64),
}

impl GraphInit {
    pub fn new(width: i32, height: i32, color: (f64, f64, f64)) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

const MAX_POINTS: usize = 300;
const OFFSET: f64 = 10.0;

#[derive(Debug)]
pub struct Graph {
    /// width of DrawingArea
    width: i32,
    /// height of DrawingArea
    height: i32,
    /// max of points rounded up to the nearest 0.5
    max_y: f64,
    /// mean calculated from points
    current_mean: f64,
    /// points on graph
    points: VecDeque<f64>,
    /// rgb 0.0 to 1.0
    color: (f64, f64, f64),
    handler: DrawHandler,
}

impl Graph {
    fn draw(&self, cx: &Context) {
        let height = self.height as f64;
        let width = self.width as f64;
        let (red, green, blue) = self.color;

        // Clear
        cx.set_operator(Operator::Clear);
        cx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cx.paint().expect("Failed to clear buffer");

        cx.set_operator(Operator::Add);

        // Background
        cx.set_source_rgba(1.0, 1.0, 1.0, 0.05);
        cx.paint().expect("Failed to paint background");

        // Graph lines
        cx.set_line_width(2.0);

        // Scale
        let x_scale = (width - 2.0 * OFFSET) / (MAX_POINTS as f64 + 1.0);
        let y_scale = (height - 3.0 * OFFSET) / self.max_y;

        // Mean line
        cx.set_source_rgba(1.0, 1.0, 1.0, 0.15);
        cx.set_dash(&[4.0, 4.0], 0.0);
        cx.move_to(OFFSET, height - (OFFSET + y_scale * self.current_mean));
        cx.line_to(
            width - OFFSET,
            height - (OFFSET + y_scale * self.current_mean),
        );
        cx.stroke().expect("Failed to draw mean line");

        cx.set_dash(&[], 0.0); // Undash line

        // Draw Graph
        cx.move_to(width - OFFSET, height - OFFSET);

        for (i, point) in self.points.iter().enumerate() {
            let x = width - (OFFSET + x_scale * (i as f64 + 1.0));
            let y = height - OFFSET - point * y_scale;

            cx.line_to(x, y);
        }

        cx.set_source_rgba(red, green, blue, 0.2);
        cx.fill_preserve().expect("Failed to fill under graph");

        cx.set_source_rgba(red, green, blue, 1.0);
        cx.stroke().expect("Failed to draw graph line");

        // AA
        cx.set_antialias(gtk::cairo::Antialias::Good);
    }
}

#[derive(Debug)]
pub enum GraphMsg {
    PushPoint(f64),
    PushPoints(Vec<f64>),
    SetColor((f64, f64, f64)),
    Clear,
}

#[derive(Debug)]
pub struct UpdateGraphMsg;

#[relm4::component(pub, async)]
impl AsyncComponent for Graph {
    type Init = GraphInit;
    type Input = GraphMsg;
    type Output = ();
    type CommandOutput = UpdateGraphMsg;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            #[local_ref]
            area -> gtk::DrawingArea {
                set_content_width: model.width,
                set_content_height: model.height,
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Graph {
            width: init.width,
            height: init.height,
            max_y: 0.0,
            current_mean: 0.0,
            points: VecDeque::from_iter(vec![0.0; MAX_POINTS]),
            color: init.color,
            handler: DrawHandler::new(),
        };

        let area = model.handler.drawing_area();
        let widgets = view_output!();

        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    loop {
                        sleep(Duration::from_millis(20)).await;
                        out.send(UpdateGraphMsg).unwrap();
                    }
                })
                .drop_on_shutdown()
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            GraphMsg::PushPoint(p) => {
                self.points.pop_back();
                self.points.push_front(p);
            }
            GraphMsg::PushPoints(ps) => {
                for p in ps {
                    self.points.pop_back();
                    self.points.push_front(p);
                }
            }
            GraphMsg::SetColor((r, g, b)) => {
                self.color = (r, g, b);
            }
            GraphMsg::Clear => {
                self.points = VecDeque::from_iter(vec![0.0; MAX_POINTS]);
            }
        }

        // Calculate and update current_mean
        let (sum, count) = self
            .points
            .iter()
            .filter(|&&x| x != 0.0)
            .fold((0.0, 0), |(sum, count), &x| (sum + x, count + 1));
        self.current_mean = if sum > 0.0 { sum / count as f64 } else { 0.0 };

        // Calculate and update max_y
        let max = self.points.iter().fold(f64::MIN, |a, &b| a.max(b));
        self.max_y = if max > 0.0 {
            // Round up to the nearest 0.5
            (max * 2.0).round() / 2.0
        } else {
            1.0
        };

        // Draw context
        let cx = self.handler.get_context();
        self.draw(&cx);
    }

    async fn update_cmd(
        &mut self,
        _: UpdateGraphMsg,
        _: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        // Draw context
        let cx = self.handler.get_context();
        self.draw(&cx);
    }
}
