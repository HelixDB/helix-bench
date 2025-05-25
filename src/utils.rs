use rand::Rng;

pub fn generate_random_vectors(count: usize, dims: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..count * dims)
        .map(|_| rng.gen_range(0.0..1.0))
        .collect()
}
