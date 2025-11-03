use try1::NeuralNetwork;
use try1::make_rng_from_args;
use try1::rng_name_from_args;

fn main() {
    // Example dimensions for the 3D grid:
    // x = layers, y = nodes per layer, z = weights per node
    let x_layers = 3usize;
    let y_nodes = 4usize;
    let z_weights = 2usize;

    // Choose RNG by CLI: --rng=default|mulberry64|pcg64|chacha8 and optional --seed=<u64>
    let rng_label = rng_name_from_args(std::env::args().skip(1));
    let mut rng = make_rng_from_args(std::env::args().skip(1));

    // Initialize with random values in [-1.0, 1.0] via selected RNG (f64)
    let nn: NeuralNetwork<f64> = NeuralNetwork::random_uniform_f64_with(x_layers, y_nodes, z_weights, -1.0f64, 1.0f64, &mut *rng);

    println!("RNG: {}", rng_label);
    let (x, y, z) = nn.dims();
    println!("NeuralNetwork<f64> dims: x(layers)={}, y(nodes)={}, z(weights)={} | total elements={}",
        x, y, z, nn.len());

    // Print a small sample to verify
    for layer in 0..x.min(2) {
        for node in 0..y.min(2) {
            for weight in 0..z.min(2) {
                if let Some(val) = nn.get(layer, node, weight) {
                    println!("nn[{}, {}, {}] = {}", layer, node, weight, val);
                }
            }
        }
    }
}
