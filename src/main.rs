use dioxus::prelude::*;
use dioxus_logger::tracing::{debug, error, info, Level};
use gloo_utils::format::JsValueSerdeExt;
use moonlight::ml::clustering::kmeans::KMeans;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde_json::json;
use std::f64;
use web_sys::js_sys;

use serde_json::Value;
use wasm_bindgen::prelude::*;

// Vega-Embed JavaScript bindings
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn vegaEmbed(selector: &str, spec: &JsValue, opt: &JsValue) -> js_sys::Promise;
}

// Helper function to create a layer specification
fn create_layer_spec(mark_type: &str, encoding: Value, transform: Option<Value>) -> Value {
    let mut layer = json!({
        "mark": mark_type,
        "encoding": encoding,
    });

    if let Some(transform_spec) = transform {
        layer
            .as_object_mut()
            .unwrap()
            .insert("transform".to_string(), transform_spec);
    }

    layer
}

// Helper function to create point encoding
fn point_encoding(x_field: &str, y_field: &str, color_field: Option<&str>) -> Value {
    let mut encoding = json!({
        "x": {
            "field": x_field,
            "type": "quantitative",
            "scale": {"zero": false}
        },
        "y": {
            "field": y_field,
            "type": "quantitative",
            "scale": {"zero": false}
        },
        "tooltip": [
            {"field": x_field, "type": "quantitative", "format": ".2f"},
            {"field": y_field, "type": "quantitative", "format": ".2f"}
        ]
    });

    if let Some(color) = color_field {
        encoding.as_object_mut().unwrap().insert(
            "color".to_string(),
            json!({
                "field": color,
                "type": "nominal"
            }),
        );

        // Add color field to tooltip if it exists
        if let Some(tooltips) = encoding.get_mut("tooltip").and_then(|t| t.as_array_mut()) {
            tooltips.push(json!({
                "field": color,
                "type": "nominal"
            }));
        }
    }

    encoding
}

// Create complete Vega-Lite specification
fn create_vega_spec(
    data: Vec<Value>,
    layers: Vec<Value>,
    width: u32,
    height: u32,
    title: &str,
) -> Value {
    json!({
        "$schema": "https://vega.github.io/schema/vega-lite/v5.json",
        "title": title,
        "width": width,
        "height": height,
        "data": {
            "values": data
        },
        "layer": layers,
        "config": {
            "view": {"stroke": null},
            "axis": {"grid": true}
        }
    })
}

#[component]
fn VegaLiteChart(
    data: Signal<Vec<Value>>,
    x_field: String,
    y_field: String,
    color_field: Option<String>,
    title: String,
    id: String,
) -> Element {
    let id_clone = id.clone();
    let spec_data = data.read().clone();

    // Create point layer
    let point_layer = create_layer_spec(
        "point",
        point_encoding(&x_field, &y_field, color_field.as_deref()),
        None,
    );

    // Create complete specification
    let spec = create_vega_spec(spec_data, vec![point_layer], 400, 200, &title);

    let spec_js = JsValue::from_serde(&spec).unwrap();
    let opt_js = JsValue::from_serde(&json!({})).unwrap();

    let selector = format!("#{}", id_clone);
    wasm_bindgen_futures::spawn_local(async move {
        let promise = vegaEmbed(&selector, &spec_js, &opt_js);
        match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(_) => info!("Vega-Lite chart embedded successfully"),
            Err(e) => error!("Error embedding Vega-Lite chart: {:?}", e),
        }
    });

    rsx! {
        div {
            id: "{id}",
            class: "w-full h-[600px] border rounded shadow-lg"
        }
    }
}

#[derive(Clone, Debug)]
struct Point {
    x: f64,
    y: f64,
}

struct ClusterParams {
    center_x: f64,
    center_y: f64,
    std_dev_x: f64,
    std_dev_y: f64,
    size: usize,
}

fn generate_realistic_clusters(
    n_clusters: usize,
    total_points: usize,
    range: f64,
) -> Vec<Vec<Point>> {
    let mut rng = rand::thread_rng();

    // Ensure we have at least 1 point per cluster
    let min_points_per_cluster = 1;
    let remaining_points = if total_points > n_clusters * min_points_per_cluster {
        total_points - (n_clusters * min_points_per_cluster)
    } else {
        0
    };

    // Generate random cluster parameters
    let cluster_params: Vec<ClusterParams> = (0..n_clusters)
        .map(|_| {
            let center_x = rng.gen_range(-range..range);
            let center_y = rng.gen_range(-range..range);
            let std_dev_x = rng.gen_range(0.3..2.0);
            let std_dev_y = rng.gen_range(0.3..2.0);

            // Ensure each cluster gets at least one point
            let extra_points = if remaining_points > 0 {
                let base = (remaining_points / n_clusters) as i64;
                let variation = std::cmp::max(1, base / 4) as i64;
                rng.gen_range(-variation..=variation) + base
            } else {
                0
            } as usize;

            // avoid stack overflow
            if extra_points > 1000 {
                error!("Extra points: {}", extra_points);
                return ClusterParams {
                    center_x,
                    center_y,
                    std_dev_x,
                    std_dev_y,
                    size: 0,
                };
            }

            debug!(
                "Min points per cluster: {}, extra points: {}",
                min_points_per_cluster, extra_points
            );
            let size = min_points_per_cluster + extra_points;

            ClusterParams {
                center_x,
                center_y,
                std_dev_x,
                std_dev_y,
                size,
            }
        })
        .collect();

    // Generate points for each cluster
    cluster_params
        .iter()
        .map(|cluster| {
            let normal_x = match Normal::new(cluster.center_x, cluster.std_dev_x) {
                Ok(normal) => normal,
                Err(err) => {
                    error!("Error creating normal distribution for x: {:?}", err);
                    return vec![];
                }
            };
            let normal_y = match Normal::new(cluster.center_y, cluster.std_dev_y) {
                Ok(normal) => normal,
                Err(err) => {
                    error!("Error creating normal distribution for y: {:?}", err);
                    return vec![];
                }
            };

            (0..cluster.size)
                .map(|_| Point {
                    x: normal_x.sample(&mut rng),
                    y: normal_y.sample(&mut rng),
                })
                .collect()
        })
        .collect()
}

#[component]
fn KMeansComponent(k: usize, max_iter: usize, tolerance: f64) -> Element {
    // state
    // data
    let mut num_points = use_signal(|| 10);
    let mut n_clusters = use_signal(|| 2);
    let mut vega_data = use_signal(|| vec![]);
    let mut k = use_signal(|| k);
    let mut max_iter = use_signal(|| max_iter);
    let mut tolerance = use_signal(|| tolerance);
    let model = KMeans::new(*k.read());

    // Convert cluster points to Vega-Lite compatible format
    use_effect(move || {
        let data = {
            let clusters =
                generate_realistic_clusters(*n_clusters.read(), *num_points.read(), 10.0);
            let data: Vec<_> = clusters
                .iter()
                .enumerate()
                .flat_map(|(cluster_idx, points)| {
                    points.iter().map(move |point| {
                        json!({
                            "x": point.x,
                            "y": point.y,
                            "cluster": format!("Cluster {}", cluster_idx)
                        })
                    })
                })
                .collect();

            // Debug: Log the first few data points
            if !data.is_empty() {
                web_sys::console::log_1(&format!("First data point: {:?}", data[0]).into());
                web_sys::console::log_1(&format!("Total points: {}", data.len()).into());
            } else {
                web_sys::console::log_1(&"No data points generated".into());
            }

            data
        };
        vega_data.set(data);
    });

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
                    "Number of points: "
                    input {
                        type: "number",
                        name: "num_points",
                        placeholder: "Number of points",
                        value: num_points,
                        min: "0",
                        onchange: move |event| {
                            let value = event.value().parse();
                            match value {
                                Ok(value) => num_points.set(value),
                                Err(err) => error!("Error parsing number of points: {:?}", err),
                            }
                        }
                    }
                }
                label{
                    "Number of Gaussian clusters: "
                    input {
                        type: "number",
                        value: n_clusters,
                        name: "n_clusters",
                        placeholder: "Number of clusters",
                        oninput: move |event| {
                            let value = event.value().parse();
                            match value {
                                Ok(value) => n_clusters.set(value),
                                Err(err) => error!("Error parsing number of clusters: {:?}", err),
                            }
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
                            let value = event.value().parse();
                            match value {
                                Ok(value) => k.set(value),
                                Err(err) => error!("Error parsing k: {:?}", err),
                            };
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
                            let value = event.value().parse();
                            match value {
                                Ok(value) => max_iter.set(value),
                                Err(err) => error!("Error parsing max_iter: {:?}", err),
                            };
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
                            let value = event.value().parse();
                            match value {
                                Ok(value) => tolerance.set(value),
                                Err(err) => error!("Error parsing tolerance: {:?}", err),
                            };
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

            VegaLiteChart {
                data: vega_data,
                x_field: "x".to_string(),
                y_field: "y".to_string(),
                color_field: Some("cluster".to_string()),
                title: "KMeans Clustering".to_string(),
                id: "kmeans_chart".to_string()
            }
        }
    }
}

fn main() {
    dioxus_logger::init(Level::DEBUG).expect("logger failed to init");
    info!("Starting KMeans example");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            KMeansComponent { k: 5, max_iter: 100, tolerance: 1e-4 }
        }
    }
}
