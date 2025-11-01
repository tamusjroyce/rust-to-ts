use try1::NeuralNetwork;

fn main() {
    // Example dimensions for the 3D grid:
    // x = layers, y = nodes per layer, z = weights per node
    let x_layers = 3usize;
    let y_nodes = 4usize;
    let z_weights = 2usize;

    // Initialize with random values in [-1.0, 1.0]
    let nn: NeuralNetwork<f32> = NeuralNetwork::random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);

    let (x, y, z) = nn.dims();
    println!("NeuralNetwork<f32> dims: x(layers)={}, y(nodes)={}, z(weights)={} | total elements={}",
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
