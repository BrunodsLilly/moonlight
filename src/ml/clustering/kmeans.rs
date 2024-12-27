#[derive(Debug, Clone)]
pub struct KMeans {
    pub k: usize,
    pub max_iter: usize,
    pub tolerance: f64,
}

impl KMeans {
    pub fn to_string(&self) -> String {
        format!(
            "KMeans{{ k: {}, max_iter: {}, tolerance: {} }}",
            self.k, self.max_iter, self.tolerance
        )
    }
}
