//! Simple, generic 3D neural network grid: layers (x), nodes per layer (y), and weights per node (z).
//! Designed to be easy to traverse for future 3D/isometric rendering.

use rand::distributions::{uniform::SampleUniform, Distribution, Uniform};

/// A compact, flat 3D grid representing a neural network structure.
///
/// Dimensions:
/// - x: number of layers
/// - y: number of nodes per layer
/// - z: number of weights per node
///
/// Storage is laid out as [x][y][z] in a single Vec, with index formula:
/// idx = x * (y*z) + y * z + z
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeuralNetwork<T> {
    x_layers: usize,
    y_nodes: usize,
    z_weights: usize,
    data: Vec<T>,
}

impl<T> NeuralNetwork<T> {
    /// Create a new network filled with a cloned default value.
    pub fn new_with_value(x_layers: usize, y_nodes: usize, z_weights: usize, value: T) -> Self
    where
        T: Clone,
    {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");
        Self {
            x_layers,
            y_nodes,
            z_weights,
            data: vec![value; size],
        }
    }

    /// Dimensions as (x_layers, y_nodes, z_weights)
    #[inline]
    pub fn dims(&self) -> (usize, usize, usize) {
        (self.x_layers, self.y_nodes, self.z_weights)
    }

    /// Total number of scalar elements (x * y * z).
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if there are no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Calculate flat index from 3D coordinates, if in-bounds.
    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < self.x_layers && y < self.y_nodes && z < self.z_weights {
            let yz = self.y_nodes * self.z_weights;
            Some(x * yz + y * self.z_weights + z)
        } else {
            None
        }
    }

    /// Get immutable reference by 3D coordinate.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        self.index(x, y, z).map(|i| &self.data[i])
    }

    /// Get mutable reference by 3D coordinate.
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut T> {
        self.index(x, y, z).map(move |i| &mut self.data[i])
    }

    /// Visit every element with its 3D coordinates.
    pub fn for_each<F: FnMut(usize, usize, usize, &T)>(&self, mut f: F) {
        let (x_max, y_max, z_max) = self.dims();
        for x in 0..x_max {
            for y in 0..y_max {
                for z in 0..z_max {
                    // Safety: checked bounds via loops
                    let idx = x * (y_max * z_max) + y * z_max + z;
                    f(x, y, z, &self.data[idx]);
                }
            }
        }
    }

    /// Mutable visit for every element with its 3D coordinates.
    pub fn for_each_mut<F: FnMut(usize, usize, usize, &mut T)>(&mut self, mut f: F) {
        let (x_max, y_max, z_max) = self.dims();
        for x in 0..x_max {
            for y in 0..y_max {
                for z in 0..z_max {
                    let idx = x * (y_max * z_max) + y * z_max + z;
                    f(x, y, z, &mut self.data[idx]);
                }
            }
        }
    }
}

impl<T> NeuralNetwork<T>
where
    T: Copy + SampleUniform,
{
    /// Create a network filled with random values uniformly sampled in [low, high].
    /// Requires T types supported by rand's Uniform (e.g., f32, f64, integers).
    pub fn random_uniform(
        x_layers: usize,
        y_nodes: usize,
        z_weights: usize,
        low: T,
        high: T,
    ) -> Self {
        let size = x_layers
            .checked_mul(y_nodes)
            .and_then(|v| v.checked_mul(z_weights))
            .expect("dimensions too large (overflow)");

    let mut rng = rand::thread_rng();
    let dist = Uniform::new(low, high);
        let mut data = Vec::with_capacity(size);
        for _ in 0..size {
            data.push(dist.sample(&mut rng));
        }

        Self {
            x_layers,
            y_nodes,
            z_weights,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NeuralNetwork;

    #[test]
    fn dims_and_indexing() {
        let mut nn = NeuralNetwork::new_with_value(2, 3, 4, 0i32);
        assert_eq!(nn.dims(), (2, 3, 4));
        assert_eq!(nn.len(), 2 * 3 * 4);

        // Write specific coordinates and read back
        *nn.get_mut(1, 2, 3).unwrap() = 42;
        assert_eq!(*nn.get(1, 2, 3).unwrap(), 42);

        // Out-of-bounds
        assert!(nn.get(2, 0, 0).is_none());
        assert!(nn.get(0, 3, 0).is_none());
        assert!(nn.get(0, 0, 4).is_none());
    }
}
