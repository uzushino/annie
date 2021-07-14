use serde::Serialize;

pub mod euclidean;
//pub mod hamming;

//pub mod manhattan;
pub use euclidean::Euclidean;
//pub use hamming::Hamming;

fn get_norm<const N: usize>(v: [f64; N]) -> f64 {
    let mut sq_norm = 0.0;

    for z in 0..N {
        sq_norm += v[z as usize] * v[z as usize];
    }

    sq_norm.sqrt()
}

fn normalize<T: num::Num + num::ToPrimitive + num::FromPrimitive + Copy, const N: usize>(
    v: [T; N],
) -> [T; N] {
    let nv = to_f64_slice(v);
    let norm = get_norm(nv);

    let mut v2 = [T::zero(); N];
    for z in 0..N {
        v2[z] = T::from_f64(nv[z] / norm).unwrap();
    }

    v
}

const ITERATION_STEPS: usize = 200;

pub fn to_f64_slice<T: num::ToPrimitive + Copy, const N: usize>(v: [T; N]) -> [f64; N] {
    let mut c = [0.0; N];

    for (z, it) in v.iter().enumerate().take(N) {
        c[z] = it.to_f64().unwrap_or_default();
    }

    c
}

fn two_means<T: num::Num + num::ToPrimitive + num::FromPrimitive + Copy, D: Distance<T>>(nodes: Vec<D::Node>, f: usize) -> (Vec<f64>, Vec<f64>) {
    let count = nodes.len();
    let i: u64 = rand::random::<u64>() % count as u64;
    let mut j: u64 = rand::random::<u64>() % (count - 1) as u64;
    j += (j >= i) as u64;
    let mut iv = to_f64_slice(nodes[i as usize].vector());
    let mut jv = to_f64_slice(nodes[j as usize].vector());

    let mut ic = 1.0;
    let mut jc = 1.0;

    for _ in 0..ITERATION_STEPS {
        let k = rand::random::<usize>() % count as usize;
        let di = ic * D::distance(nodes[i as usize].vector(), nodes[k].vector(), f);
        let dj = jc * D::distance(nodes[j as usize].vector(), nodes[k].vector(), f);
        let nk = to_f64_slice(nodes[k].vector());

        if di < dj {
            for z in 0..f {
                let v = iv[z] * ic + nk[z];
                iv[z] = v / (ic + 1.0);
            }

            ic += 1.0;
        } else if dj < di {
            for z in 0..f {
                let v = jv[z] * jc + nk[z];
                jv[z] = v / (jc + 1.0);
            }

            jc += 1.0;
        }
    }

    (iv, jv)
}

pub trait NodeImpl<T> {
    fn new() -> Self;

    fn reset(&mut self, w: Vec<T>);
    fn copy(&mut self, other: Self);

    fn descendant(&self) -> usize;
    fn set_descendant(&mut self, other: usize);

    fn vector(&self) -> Vec<T>;
    fn set_vector(&self, _other: Vec<T>) {}

    fn children(&self) -> Vec<i64>;
    fn set_children(&mut self, other: Vec<i64>);
}

pub trait Distance<T> {
    type Node: NodeImpl<T> + Clone + Serialize;

    fn distance(x: Vec<T>, y: Vec<T>, f: usize) -> f64;

    fn create_split(nodes: Vec<Self::Node>, n: &mut Self::Node, f: usize);

    fn side(n: &Self::Node, y: Vec<T>) -> bool;

    fn margin(n: &Self::Node, y: Vec<T>) -> f64;

    fn normalized_distance(distance: f64) -> f64;
}
