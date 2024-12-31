const EPSILON: f64 = 1e-6;
const MAX_ITER: usize = 300;

// Point in n-dimensional space
type Point = Vec<f64>;

#[derive(Debug, Clone)]
pub struct KMeans {
    pub k: usize,
    pub max_iter: usize,
    pub tolerance: f64,

    // Current state
    centroids: Vec<Point>,
    assignments: Vec<usize>,
    inertia: f64,
    n_iter: usize,
    converged: bool,
}

impl KMeans {
    pub fn to_string(&self) -> String {
        format!(
            "KMeans{{ k: {}, max_iter: {}, tolerance: {} }}",
            self.k, self.max_iter, self.tolerance
        )
    }

    pub fn new(k: usize) -> Self {
        KMeans {
            k,
            max_iter: MAX_ITER,
            tolerance: EPSILON,
            centroids: vec![],
            assignments: vec![],
            inertia: 0.0,
            n_iter: 0,
            converged: false,
        }
    }

    // Core functionality
    fn step(&mut self, data: &Vec<Point>) {}
}
