// Converted from Rust: fn main(...)
function main(): void {
  // Rust variable declaration
  const x_layers = 3;
  // Rust variable declaration
  const y_nodes = 4;
  // Rust variable declaration
  const z_weights = 2;
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  const [x, y, z] = nn.dims() as any;
  // Rust macro
  console.log("NeuralNetwork<f32> dims: x(layers)={}, y(nodes)={}, z(weights)={} | total elements={}" , x , y , z , nn . len ());
  // Unsupported trailing expression
}

