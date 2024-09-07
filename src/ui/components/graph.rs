use adw::prelude::*;
use gtk::{glib, prelude::*, subclass::prelude::*};

use relm4::prelude::*;

use plotters::{
    prelude::*,
    series::AreaSeries,
    style::{Color, RGBColor},
};
use plotters_cairo::CairoBackend;

use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    error::Error,
    sync::Arc,
};

const MAX_DATA_POINTS: usize = 300;

#[derive(Debug, Clone)]
pub struct Graph {
    pub data_points: RefCell<VecDeque<f64>>,
    pub max_y: Cell<Option<f64>>,
    pub graph_color: Cell<RGBColor>,
}

impl Default for Graph {
    fn default() -> Self {
        let mut empty = VecDeque::with_capacity(MAX_DATA_POINTS);
        for _ in 0..MAX_DATA_POINTS {
            empty.push_back(0.0);
        }

        Self {
            data_points: RefCell::new(empty),
            max_y: Cell::new(Some(1.0)),
            graph_color: Cell::new(RGBColor(250, 97, 0)),
        }
    }
}

impl Graph {
    pub fn plot_graph<'a, DB>(&self, backend: DB) -> Result<(), Box<dyn Error + 'a>>
    where
        DB: DrawingBackend + 'a,
    {
        let data_points = self.data_points.borrow();
        let color = self.graph_color.get();

        let start_point = MAX_DATA_POINTS as usize;

        let root = backend.into_drawing_area();

        root.fill(&self.graph_color.get().mix(0.1))?;

        let y_max = self.max_y.get().unwrap_or_else(|| {
            let max = *data_points
                .range(start_point..MAX_DATA_POINTS)
                .max_by(|x, y| x.total_cmp(y))
                .unwrap_or(&0.0);
            if max == 0.0 {
                f64::EPSILON
            } else {
                max
            }
        });

        let mut chart = ChartBuilder::on(&root)
            .build_cartesian_2d(0f64..MAX_DATA_POINTS as f64 - 1.0, 0f64..y_max)?;

        chart.draw_series(
            AreaSeries::new(
                (0..)
                    .zip(data_points.range(start_point..MAX_DATA_POINTS))
                    .map(|(x, y)| (x as f64, *y)),
                0.0,
                color.mix(0.4),
            )
            .border_style(color),
        )?;

        root.present()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GraphComponent {
    graph: Arc<Graph>,
}

#[derive(Clone, Debug)]
pub enum GraphComponentMsg {
    PushPoint(f64),
    PushPoints(Vec<f64>),
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GraphComponent {
    type Input = GraphComponentMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            #[local_ref]
            drawing_area -> gtk::DrawingArea {
                set_draw_func => move |_, cr, width, height| {
                    let backend = CairoBackend::new(&cr, (width as u32, height as u32)).unwrap();
                    graph.plot_graph(backend).unwrap();
                }
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = GraphComponent {
            graph: Arc::new(Graph::default()),
        };
        let drawing_area = gtk::DrawingArea::builder()
            .width_request(1000)
            .height_request(300)
            .build();
        let graph: Arc<Graph> = Arc::clone(&model.graph);

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GraphComponentMsg::PushPoint(point) => {
                let mut graph = self.graph.data_points.borrow_mut();
                graph.pop_back();
                graph.push_front(point);
            }
            GraphComponentMsg::PushPoints(points) => {
                let mut graph = self.graph.data_points.borrow_mut();
                let mut iter = points.iter();
                while let Some(point) = iter.next() {
                    graph.pop_back();
                    graph.push_front(*point);
                }
            }
        }
    }
}
