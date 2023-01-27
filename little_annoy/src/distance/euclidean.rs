use rand::rngs::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::distance::{normalize, two_means, Distance, NodeImpl};
use crate::item::Item;

pub struct Euclidean {}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node<T: Item> {
    pub children: Vec<i64>,
    pub v: Vec<T>,
    pub n_descendants: usize,
    pub a: T,
    f: usize,
}

impl<T: Item> NodeImpl<T> for Node<T> {
    fn new(f: usize) -> Self {
        Node {
            children: vec![0, 0],
            v: vec![T::zero(); f],
            n_descendants: 0,
            a: T::zero(),
            f,
        }
    }

    fn reset(&mut self, v: &[T]) {
        self.children[0] = 0;
        self.children[1] = 0;
        self.n_descendants = 1;
        self.a = T::zero();
        self.v = v.to_vec();
        self.f = 0;
    }

    fn descendant(&self) -> usize {
        self.n_descendants
    }

    fn set_descendant(&mut self, other: usize) {
        self.n_descendants = other;
    }

    fn as_slice(&self) -> &[T] {
        self.v.as_slice()
    }

    fn mut_vector(&mut self) -> &mut Vec<T> {
        &mut self.v
    }

    fn children(&self) -> Vec<i64> {
        self.children.clone()
    }

    fn set_children(&mut self, other: Vec<i64>) {
        self.children = other;
    }

    fn copy(&mut self, other: Self) {
        self.n_descendants = other.n_descendants;
        self.children = other.children;
        self.v = other.v;
        self.a = other.a;
        self.f = other.f;
    }
}

impl<T: Item + serde::Serialize + serde::de::DeserializeOwned + std::ops::Neg + num::Num>
    Distance<T> for Euclidean
{
    type Node = Node<T>;

    #[inline]
    fn margin(n: &Self::Node, y: &[T]) -> T {
        let mut dot = n.a;

        (0..y.len()).for_each(|z| {
            let v = n.v[z as usize] * y[z as usize];
            dot += v;
        });

        dot
    }

    #[inline]
    fn side(n: &Self::Node, y: &[T], rng: &mut ThreadRng) -> bool {
        let dot = Self::margin(n, y);
        if dot != T::zero() {
            return dot > T::zero();
        }
        rng.gen()
    }

    #[inline]
    fn distance(x: &[T], y: &[T], f: usize) -> T {
        let mut d = T::zero();

        for i in 0..f {
            let v = (x[i as usize] - y[i as usize]) * (x[i as usize] - y[i as usize]);
            d += v;
        }

        d
    }

    #[inline]
    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }

    #[inline]
    fn create_split(nodes: &[&Self::Node], n: &mut Self::Node, f: usize, rng: &mut ThreadRng) {
        let (best_iv, best_jv) = two_means::<T, Euclidean>(rng, nodes, f);

        for z in 0..f {
            let best = best_iv[z] - best_jv[z];
            n.v[z] = best;
        }

        n.v = normalize(&n.v);
        n.a = T::zero();

        for z in 0..f {
            let v = -n.v[z] * (best_iv[z] + best_jv[z]) / (T::one() + T::one());
            n.a += v;
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;

    use super::*;

    #[test]
    fn test_distance() {
        let x = &[1.0, 2.0];
        let y = &[2.0, 4.0];
        let f = 2;

        let dist = Euclidean::distance(x, y, f);

        assert_eq!(dist, 5.0);
    }

    #[test]
    fn test_side() {
        let mut n = Node::new(2);
        n.v = vec![2., 4.];
        let actual = Euclidean::side(&n, &[1., 2.], &mut thread_rng());

        assert!(actual)
    }
}
