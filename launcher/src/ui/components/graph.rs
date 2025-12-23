// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::VecDeque;
use std::time::Duration;

use adw::prelude::*;

use relm4::prelude::*;
use relm4::abstractions::{DrawContext, DrawHandler};

use agl_core::export::tasks::tokio;

const OFFSET: f64 = 8.0;
const UPDATE_INTERVAL: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, Copy, PartialEq)]
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

        // Draw the mean line.
        context.set_source_rgba(1.0, 1.0, 1.0, 0.15);
        context.set_dash(&[4.0, 4.0], 0.0);

        context.move_to(OFFSET, height - (OFFSET + y_scale * self.mean_point as f64));
        context.line_to(width - OFFSET, height - (OFFSET + y_scale * self.mean_point as f64));

        context.stroke()?;

        // Return solid line style.
        context.set_dash(&[], 0.0);

        // Collect all points.
        let mut plot_points = Vec::new();

        plot_points.push((
            width - OFFSET,
            height - (OFFSET + self.points[0] as f64 * y_scale)
        ));

        for (i, point) in self.points.iter().enumerate() {
            let x = width - (OFFSET + x_scale * (i + 1) as f64);
            let y = height - OFFSET - *point as f64 * y_scale;

            plot_points.push((x, y));
        }

        plot_points.push((
            OFFSET,
            height - (OFFSET + self.points[self.points.len() - 1] as f64 * y_scale)
        ));

        fn draw_curve(context: &DrawContext, points: &[(f64, f64)]) {
            // Draw smooth Catmull-Rom spline through points.
            for i in 0..points.len() - 1 {
                let p0 = if i == 0 { points[0] } else { points[i - 1] };
                let p1 = points[i];
                let p2 = points[i + 1];
                let p3 = points.get(i + 2).unwrap_or(&points[i + 1]);

                // Convert to Bézier.
                context.curve_to(
                    p1.0 + (p2.0 - p0.0) / 6.0,
                    p1.1 + (p2.1 - p0.1) / 6.0,
                    p2.0 - (p3.0 - p1.0) / 6.0,
                    p2.1 - (p3.1 - p1.1) / 6.0,
                    p2.0,
                    p2.1
                );
            }
        }

        if plot_points.len() > 1 {
            let baseline = height - OFFSET;

            // Draw filled area.
            context.move_to(plot_points[0].0, baseline);
            context.line_to(plot_points[0].0, plot_points[0].1);

            draw_curve(&context, &plot_points);

            context.line_to(plot_points[plot_points.len() - 1].0, baseline);
            context.close_path();

            context.set_source_rgba(red, green, blue, 0.2);
            context.fill()?;

            // Draw smooth curve.
            context.move_to(plot_points[0].0, plot_points[0].1);

            draw_curve(&context, &plot_points);

            context.set_source_rgba(red, green, blue, 1.0);
            context.stroke()?;
        }

        // Enable antialiasing.
        context.set_antialias(gtk::cairo::Antialias::Good);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GraphMsg {
    SetColor {
        red: f64,
        green: f64,
        blue: f64
    },
    AddPoint(u64),
    Clear
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphUpdateMsg;

#[relm4::component(pub, async)]
impl AsyncComponent for Graph {
    type Init = GraphInit;
    type Input = GraphMsg;
    type Output = ();
    type CommandOutput = GraphUpdateMsg;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "card",

            #[local_ref]
            _area -> gtk::DrawingArea {
                set_content_width: model.width,
                set_content_height: model.height
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
            window_size: init.window_size,
            color: init.color,

            max_point: 0,
            mean_point: 0,

            points: VecDeque::from_iter(vec![0; init.window_size]),
            handler: DrawHandler::new()
        };

        let _area = model.handler.drawing_area();

        let widgets = view_output!();

        sender.command(|sender, shutdown| {
            shutdown
                .register(async move {
                    while sender.send(GraphUpdateMsg).is_ok() {
                        tokio::time::sleep(UPDATE_INTERVAL).await;
                    }
                })
                .drop_on_shutdown()
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root
    ) {
        match msg {
            GraphMsg::SetColor { red, green, blue } => self.color = (red, green, blue),

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
    }

    async fn update_cmd(
        &mut self,
        _msg: Self::CommandOutput,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root
    ) {
        if let Err(err) = self.draw() {
            tracing::error!(?err, "failed to draw graph on update tick");
        }
    }
}
