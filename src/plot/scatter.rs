use dioxus::prelude::*;
use std::f64;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

pub trait Drawable {
    fn draw(&self, context: &CanvasRenderingContext2d, width: f64, height: f64);
}

#[derive(Clone)]
pub struct ScatterPlotData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl Drawable for ScatterPlotData {
    fn draw(&self, context: &CanvasRenderingContext2d, width: f64, height: f64) {
        let margin = 50.0;

        // Determine data ranges
        let x_min = self.x.iter().cloned().reduce(f64::min).unwrap_or(0.0);
        let x_max = self.x.iter().cloned().reduce(f64::max).unwrap_or(1.0);
        let y_min = self.y.iter().cloned().reduce(f64::min).unwrap_or(0.0);
        let y_max = self.y.iter().cloned().reduce(f64::max).unwrap_or(1.0);

        // Draw axes
        draw_axes(context, width, height, self.x.clone(), self.y.clone());

        // Draw data points
        let x_scale = (width - 2.0 * margin) / (x_max - x_min);
        let y_scale = (height - 2.0 * margin) / (y_max - y_min);

        for i in 0..self.x.len() {
            let x_pos = margin + (self.x[i] - x_min) * x_scale;
            let y_pos = height - margin - (self.y[i] - y_min) * y_scale;

            context.begin_path();
            context
                .arc(x_pos, y_pos, 5.0, 0.0, 2.0 * f64::consts::PI)
                .unwrap_or_else(|_| ());
            context.fill();
        }
    }
}

#[component]
pub fn ScatterPlot(x: Signal<Vec<f64>>, y: Signal<Vec<f64>>, width: f64, height: f64) -> Element {
    let canvas_id = "scatterplot_canvas";

    use_effect(move || {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();
        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        // Clear the canvas
        clear_canvas(&context, width, height);

        // Draw axes
        draw_axes(&context, width, height, x.read().clone(), y.read().clone());

        // Plot points
        draw_points(&context, &x.read(), &y.read(), width, height);
    });

    rsx! {
        div {
            canvas {
                id: "{canvas_id}",
                width: "{width}",
                height: "{height}",
                style: "border: 1px solid black;"
            }
        }
    }
}

fn clear_canvas(context: &CanvasRenderingContext2d, width: f64, height: f64) {
    context.clear_rect(0.0, 0.0, width, height);
}
fn draw_axes(
    context: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    x: Vec<f64>,
    y: Vec<f64>,
) {
    let margin = 50.0;

    // Determine data ranges
    let x_min = x.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let x_max = x.iter().cloned().reduce(f64::max).unwrap_or(1.0);
    let y_min = y.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let y_max = y.iter().cloned().reduce(f64::max).unwrap_or(1.0);

    // Draw x-axis
    context.set_line_width(2.0);
    context.begin_path();
    context.move_to(margin, height - margin);
    context.line_to(width - margin, height - margin);
    context.stroke();

    // Draw y-axis
    context.begin_path();
    context.move_to(margin, margin);
    context.line_to(margin, height - margin);
    context.stroke();

    // Add x-axis ticks
    let tick_count = 10;
    let x_range = x_max - x_min;
    let y_range = y_max - y_min;

    for i in 0..=tick_count {
        let t = i as f64 / tick_count as f64;
        let x_val = x_min + t * x_range;
        let x_pos = margin + t * (width - 2.0 * margin);

        context.begin_path();
        context.move_to(x_pos, height - margin);
        context.line_to(x_pos, height - margin + 5.0);
        context.stroke();

        context.set_font("10px sans-serif");
        context
            .fill_text(
                &format!("{:.1}", x_val),
                x_pos - 10.0,
                height - margin + 20.0,
            )
            .unwrap_or_else(|_| ());
    }

    // Add y-axis ticks
    for i in 0..=tick_count {
        let t = i as f64 / tick_count as f64;
        let y_val = y_min + t * y_range;
        let y_pos = height - margin - t * (height - 2.0 * margin);

        context.begin_path();
        context.move_to(margin - 5.0, y_pos);
        context.line_to(margin, y_pos);
        context.stroke();

        context
            .fill_text(&format!("{:.1}", y_val), margin - 30.0, y_pos + 3.0)
            .unwrap_or_else(|_| ());
    }
}
fn draw_points(context: &CanvasRenderingContext2d, x: &[f64], y: &[f64], width: f64, height: f64) {
    let margin = 50.0;

    // Determine data ranges
    let x_min = x.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let x_max = x.iter().cloned().reduce(f64::max).unwrap_or(1.0);
    let y_min = y.iter().cloned().reduce(f64::min).unwrap_or(0.0);
    let y_max = y.iter().cloned().reduce(f64::max).unwrap_or(1.0);

    let x_scale = (width - 2.0 * margin) / (x_max - x_min);
    let y_scale = (height - 2.0 * margin) / (y_max - y_min);

    for i in 0..x.len() {
        let x_pos = margin + (x[i] - x_min) * x_scale;
        let y_pos = height - margin - (y[i] - y_min) * y_scale;

        context.begin_path();
        context
            .arc(x_pos, y_pos, 5.0, 0.0, 2.0 * f64::consts::PI)
            .unwrap_or_else(|_| ());
        context.fill();
    }
}
