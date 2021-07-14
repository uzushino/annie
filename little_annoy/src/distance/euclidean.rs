use num::ToPrimitive;
use serde::{Deserialize, Serialize};

use crate::distance::{normalize, two_means, Distance, NodeImpl};
use crate::random_flip;

pub struct Euclidean {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub children: Vec<i64>,
    #[serde(with = "serde_arrays")]
    pub v: Vec<f64>,
    pub n_descendants: usize,
    pub a: f64,
}

impl NodeImpl<f64> for Node {
    fn new() -> Self {
        Node {
            children: vec![0, 0],
            v: vec![0.0],
            n_descendants: 0,
            a: 0.,
        }
    }

    fn reset(&mut self, v: Vec<f64>) {
        self.children[0] = 0;
        self.children[1] = 0;
        self.n_descendants = 1;
        self.v = v;
    }

    fn descendant(&self) -> usize {
        self.n_descendants
    }

    fn set_descendant(&mut self, other: usize) {
        self.n_descendants = other;
    }

    fn vector(&self) -> Vec<f64> {
        self.v
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
    }
}

impl Distance<f64> for Euclidean {
    type Node = Node;

    fn margin(n: &Self::Node, y: Vec<f64>) -> f64 {
        let mut dot = n.a;

        for z in 0..y.len() {
            let v = n.v[z as usize] * y[z as usize];
            dot += v.to_f64().unwrap_or_default();
        }

        dot
    }

    fn side(n: &Self::Node, y: Vec<f64>) -> bool {
        let dot = Self::margin(n, y);

        if dot != 0.0 {
            return dot > 0.0;
        }

        random_flip()
    }

    fn distance(x: Vec<f64>, y: Vec<f64>, f: usize) -> f64 {
        let mut d = 0.0;

        for i in 0..f {
            let v = (x[i as usize] - y[i as usize]) * (x[i as usize] - y[i as usize]);
            d += v.to_f64().unwrap_or_default();
        }

        d
    }

    fn normalized_distance(distance: f64) -> f64 {
        distance.max(0.0).sqrt()
    }

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node, f: usize) {
        let (best_iv, best_jv) = two_means::<f64, Euclidean>(nodes, f);

        for z in 0..f {
            let best = best_iv[z] - best_jv[z];
            n.v[z] = best;
        }

        n.v = normalize(n.v);

        n.a = 0.0;

        for z in 0..f {
            let v = -n.v[z].to_f64().unwrap_or_default() * (best_iv[z] + best_jv[z]) / 2.0;
            n.a += v;
        }
    }
}
