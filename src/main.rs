use dioxus::prelude::*;
use moonlight::ml::clustering::kmeans::KMeans;
use std::f64;
use wasm_bindgen::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn ScatterPlot(x: Signal<Vec<f64>>, y: Signal<Vec<f64>>) -> Element {
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
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        // clear canvas
        context.clear_rect(0.0, 0.0, 800.0, 600.0);

        context.begin_path();

        // draw x-axis
        context.move_to(0.0, 300.0);
        context.line_to(800.0, 300.0);

        // draw y-axis
        context.move_to(400.0, 0.0);
        context.line_to(400.0, 600.0);

        context.stroke();

        // draw data points
        for i in 0..x.len() {
            context.begin_path();
            let _x = x.read();
            let _y = y.read();
            let x_val = _x.get(i).unwrap_or(&0.0);
            let y_val = _y.get(i).unwrap_or(&0.0);
            context
                .arc(
                    400.0 + x_val * 10.0,
                    300.0 - y_val * 10.0,
                    5.0,
                    0.0,
                    2.0 * f64::consts::PI,
                )
                .unwrap_or_else(|_| ());
            context.fill();
            context.stroke();
        }

        context.stroke();
    });

    rsx! {
        div {
            canvas {
                id: canvas_id,
                width: 800,
                height: 600,
            }
        }
    }
}

#[component]
fn KMeansComponent(k: usize, max_iter: usize, tolerance: f64) -> Element {
    // state
    // data
    let mut x = use_signal(|| vec![10.0, 20.0, 30.0]);
    let mut y = use_signal(|| vec![10.0, 20.0, 30.0]);
    // params
    let mut k = use_signal(|| k);
    let mut max_iter = use_signal(|| max_iter);
    let mut tolerance = use_signal(|| tolerance);
    let model = KMeans {
        k: *k.read(),
        max_iter: *max_iter.read(),
        tolerance: *tolerance.read(),
    };

    rsx! {
        div {
            h1 {
                "KMeans"
            }
            p {
                "Currently only 2D data is supported."
            }
            h4 {
                "Inputs"
            }
            fieldset {
                legend {
                    "Data"
                }
                p {
                    "Enter the x and y coordinates of your data. "
                    "Each coordinate should be separated by a comma."
                }
                label {
                    "x: "
                    input {
                        type: "text",
                        name: "x",
                        value: x.read().iter().map(|x| x.to_string()).collect::<Vec<String>>().join(","),
                        placeholder: "x",
                        onchange: move |event| {
                            x.set(event.value().split(',').map(|x| x.trim().parse().unwrap_or(0.0)).collect());
                        }
                    }
                }
                br {}
                label {
                    "y: "
                    input {
                        type: "text",
                        name: "y",
                        placeholder: "y",
                        value: y.read().iter().map(|y| y.to_string()).collect::<Vec<String>>().join(","),
                        onchange: move |event| {
                            y.set(event.value().split(',').map(|y| y.trim().parse().unwrap_or(0.0)).collect());
                        }
                    }
                }
            }
            fieldset {
                legend {
                    "Parameters"
                }
                label {
                    "k: "
                    input {
                        type: "number",
                        value: k,
                        name: "k",
                        placeholder: "k",
                        oninput: move |event| {
                            k.set(event.value().parse().unwrap());
                        }
                    }
                }
                label {
                    "max_iter: "
                    input {
                        type: "number",
                        value: max_iter,
                        name: "max_iter",
                        placeholder: "max_iter",
                        oninput: move |event| {
                            max_iter.set(event.value().parse().unwrap());
                        }
                    }
                }
                label {
                    "tolerance: "
                    input {
                        type: "number",
                        value: tolerance,
                        name: "tolerance",
                        placeholder: "tolerance",
                        oninput: move |event| {
                            tolerance.set(event.value().parse().unwrap());
                        }
                    }
                }
            }
        }
        div {
            h4 {
                "Results"
            }
            p {
                "Model: "
                { model.to_string() }
            }
            p {
                "Data: "
            }
            ScatterPlot { x, y }
        }
    }
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            h1 {
                "Hello, World!"
            }
            KMeansComponent { k: 5, max_iter: 100, tolerance: 1e-4 }
        }
    }
}
