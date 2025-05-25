use rand::Rng;

pub fn generate_random_vectors(count: usize, dims: usize) -> Vec<Vec<f64>> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(count);

    for _ in 0..count {
        let vector: Vec<f64> = (0..dims)
            .map(|_| rng.gen_range(0.0..1.0))
            .collect();
        result.push(vector);
    }

    result
}
