use crate::distance::{Distance, NodeImpl};
use crate::item::Item;
use crate::{random_flip, Numeric};

use rand::prelude::SeedableRng;
use rand::rngs::StdRng;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::usize;
use rand::Rng;
use rand::thread_rng;

use rayon::prelude::*;

#[allow(non_snake_case)]
pub struct Annoy<T: Item, D>
where
    D: Distance<T>,
{
    pub _f: usize,
    pub _K: usize,
    pub _n_nodes: i64,
    pub _n_items: i64,

    pub _nodes: HashMap<i64, D::Node>,
    pub _roots: Vec<i64>,

    pub _seed: Option<u64>,
    pub t: PhantomData<T>,
}

impl<T: Item, D: Distance<T>> Annoy<T, D> {
    pub fn new(f: usize) -> Self {
        Self {
            _roots: Vec::new(),
            _nodes: HashMap::new(),
            _n_items: 0,
            _n_nodes: 0,
            _f: f,
            _K: 6,
            _seed: None,
            t: PhantomData,
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self._seed = Some(seed);
    }

    pub fn add_item(&mut self, item: i64, w: &[T]) {
        let f = self._f;
        let n = self._nodes.entry(item).or_insert_with(|| D::Node::new(f));
        n.reset(w);

        if item >= self._n_items {
            self._n_items = item + 1;
        }
    }

    pub fn build(&mut self, q: i64) {
        self._n_nodes = self._n_items;

        loop {
            if q == -1 && self._n_nodes >= self._n_items * 2 {
                break;
            }
            if q != -1 && self._roots.len() >= (q as usize) {
                break;
            }

            let mut indices: Vec<i64> = Vec::new();
            for i in 0..self._n_items {
                if let Some(n) = self._nodes.get(&i) {
                    if n.descendant() >= 1 {
                        indices.push(i)
                    }
                }
            }

            let ind = self._make_tree(&indices);

            self._roots.push(ind);
        }
    }

    pub fn get_nns_by_vector(&self, v: &[T], n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        self._get_all_nns(v.to_vec(), n, search_k)
    }

    pub fn get_nns_by_item(&self, item: i64, n: usize, search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let m = self._nodes.get(&item).unwrap();
        let v = m.vector();

        self._get_all_nns(v.to_vec(), n, search_k)
    }

    #[cfg(not(feature="parallel_build"))]
    fn random_split_index(&self, m: &mut D::Node, indices: &[i64], children: &Vec<D::Node>) -> (Vec<i64>, Vec<i64>) {
        let mut rng = if let Some(seed) = self._seed {
            SeedableRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        D::create_split(children, m, self._f, &mut rng);

        let mut children_indices = (Vec::new(), Vec::new());

        for i in indices.iter() {
            if let Some(n) = self._nodes.get(i) {
                let side = D::side(&m, n.vector(), &mut rng);
                if side {
                    children_indices.0.push(*i);
                } else {
                    children_indices.1.push(*i);
                }
            }
        }

        while children_indices.0.is_empty() || children_indices.1.is_empty() {
            children_indices.0.clear();
            children_indices.1.clear();

            indices
                .iter()
                .for_each(|j| {
                    if random_flip(&mut rng) {
                        children_indices.0.push(*j);
                    } else {
                        children_indices.1.push(*j);
                    }
                });
        }

        children_indices
    }

    #[cfg(feature="parallel_build")]
    fn random_split_index(&self, m: &D::Node, indices: &[i64], children: &Vec<D::Node>) -> (Vec<i64>, Vec<i64>) {
        let mut rng = if let Some(seed) = self._seed {
            SeedableRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        };

        D::create_split(children, &mut m, self._f, &mut rng);

        let mut children_indices = (Vec::new(), Vec::new());

        for i in indices.iter() {
            if let Some(n) = self._nodes.get(i) {
                let side = D::side(&m, n.vector(), &mut rng);
                if side {
                    children_indices.0.push(*i);
                } else {
                    children_indices.1.push(*i);
                }
            }
        }

        while children_indices.0.is_empty() || children_indices.1.is_empty() {
            children_indices.0.clear();
            children_indices.1.clear();

            indices
                .par_iter()
                .for_each_with(|| thread_rng(), |gen, j| {
                    if gen().gen_range(0..1) == 1 {
                        children_indices.0.push(*j);
                    } else {
                        children_indices.1.push(*j);
                    }
                });
        }

        children_indices
    }

    fn _make_tree(&mut self, indices: &[i64]) -> i64 {
        if indices.len() == 1 {
            return indices[0];
        }

        let mut m = D::Node::new(self._f);

        if indices.len() <= (self._K as usize) {
            let item = self._n_nodes;
            self._n_nodes += 1;

            let m = self._nodes.entry(item).or_insert(D::Node::new(self._f));
            m.set_descendant(indices.len());
            m.set_children(indices.to_owned());

            return item;
        }

        let mut children: Vec<D::Node> = Vec::default();
        indices.iter().for_each(|index| {
            if let Some(n) = self._nodes.get(index) {
                children.push(n.clone());
            }
        });

        let children_indices = self.random_split_index(&mut m, indices, &children);

        let flip = if children_indices.0.len() > children_indices.1.len() {
            1
        } else {
            0
        };

        m.set_descendant(indices.len());

        for side in 0..2 {
            let ii = side ^ flip;
            let a = if ii == 0 {
                &children_indices.0
            } else {
                &children_indices.1
            };

            let mut v = m.children();
            v[ii] = self._make_tree(a);

            m.set_children(v);
        }

        let item = self._n_nodes;
        self._n_nodes += 1;

        let f = self._f;
        let node = self._nodes.entry(item).or_insert_with(|| D::Node::new(f));
        node.copy(m);

        item
    }

    fn _get_all_nns(&self, v: Vec<T>, n: usize, mut search_k: i64) -> (Vec<i64>, Vec<f64>)
    where
        D: Distance<T>,
    {
        let mut nodes = self._nodes.clone();
        let mut q: BinaryHeap<(Numeric<f64>, i64)> = BinaryHeap::new();
        let v = v.as_slice();
        let f = self._f;

        if search_k == -1 {
            search_k = (n as i64) * self._roots.len() as i64;
        }

        for root in self._roots.iter() {
            q.push((Numeric(0.0), *root))
        }

        let mut nns: Vec<i64> = Vec::new();

        while nns.len() < (search_k as usize) && !q.is_empty() {
            let top = q.peek().unwrap();
            let d = top.0 .0;
            let i = top.1;
            let nd = nodes.entry(i).or_insert_with(|| D::Node::new(f));

            q.pop();

            if nd.descendant() == 1 && i < self._n_items {
                nns.push(i);
            } else if nd.descendant() as usize <= self._K {
                let dst = nd.children();
                nns.extend(dst);
            } else {
                let margin = D::margin(nd, v);
                let a = T::zero() + margin;
                let b = T::zero() - margin;

                q.push((Numeric(d.min(T::to_f64(&a).unwrap())), nd.children()[1]));
                q.push((Numeric(d.min(T::to_f64(&b).unwrap())), nd.children()[0]));
            }
        }

        nns.sort_unstable();

        let mut nns_dist = Vec::new();
        let mut last = -1;

        for j in &nns {
            if *j == last {
                continue;
            }

            last = *j;
            let mut _n = nodes.entry(*j).or_insert_with(|| D::Node::new(f));
            let dist = D::distance(v, _n.vector(), self._f);
            nns_dist.push((dist, *j));
        }

        let m = nns_dist.len();
        let p = if n < m { n } else { m } as usize;

        nns_dist.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut distances = Vec::new();
        let mut result = Vec::new();

        for (dist, idx) in nns_dist.iter().take(p) {
            distances.push(D::normalized_distance(T::to_f64(dist).unwrap()));
            result.push(*idx)
        }

        (result, distances)
    }

    fn _get(&self, i: i64) -> &D::Node {
        &self._nodes[&i]
    }

    pub fn get_distance(self, i: i64, j: i64) -> f64 {
        let dist = D::distance(self._get(i).vector(), self._get(j).vector(), self._f);

        D::normalized_distance(dist.to_f64().unwrap_or(0.))
    }
}
