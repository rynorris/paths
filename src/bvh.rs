use std::time::Instant;

use crate::geom::{AABB, BoundedVolume, Collision, Primitive, Ray};
use crate::vector::Vector3;


// Ray-box collision algorithm taken from https://www.scratchapixel.com/lessons/3d-basic-rendering/minimal-ray-tracer-rendering-simple-shapes/ray-box-intersection
fn ray_box_collide(ray: &Ray, aabb: &AABB, len: Option<f64>) -> Option<f64> {
    let t0s = (aabb.min - ray.origin) * ray.inv_direction;
    let t1s = (aabb.max - ray.origin) * ray.inv_direction;
    let tsmaller = Vector3::componentwise_min(t0s, t1s);
    let tbigger = Vector3::componentwise_max(t0s, t1s);
    let tmin = tsmaller.max();
    let tmax = tbigger.min();

    if tmin < tmax && len.map_or(true, |d| tmin < d) { 
        Some(tmin)
    } else {
        None
    }
}

enum Node {
    Leaf(LeafNode),
    Cluster(ClusterNode),
}

impl Node {
    pub fn aabb(&self) -> &AABB {
        match self {
            Node::Leaf(leaf) => &leaf.aabb,
            Node::Cluster(clus) => &clus.aabb,
        }
    }
}

struct LeafNode {
    obj: usize,
    primitive: Primitive,
    aabb: AABB,
}

impl LeafNode {
    fn new(obj: usize, primitive: Primitive) -> LeafNode {
        LeafNode { obj, primitive, aabb: primitive.aabb() }
    }
}

struct ClusterNode {
    left: Box<Node>,
    right: Box<Node>,
    aabb: AABB,
}

impl ClusterNode {
    fn new(left: Box<Node>, right: Box<Node>) -> ClusterNode {
        let aabb1 = match left.as_ref() {
            Node::Leaf(leaf) => &leaf.aabb,
            Node::Cluster(clus) => &clus.aabb,
        };

        let aabb2 = match right.as_ref() {
            Node::Leaf(leaf) => &leaf.aabb,
            Node::Cluster(clus) => &clus.aabb,
        };

        let aabb = combine_aabb(&aabb1, &aabb2);
        ClusterNode { left, right, aabb }
    }
}

pub struct BVH<T> {
    items: Vec<T>,
    root: Node,
}

impl <T> BVH<T> {
    pub fn find_intersection(&self, ray: Ray) -> Option<(Collision, &T)> {
        let mut stack: [Option<&Node>; 100] = [None; 100];
        let mut stack_ptr: usize = 0;

        let mut node = if let Some(_) = ray_box_collide(&ray, &self.root.aabb(), None) {
            &self.root
        } else {
            return None;
        };

        let mut closest_collision: Option<(Collision, &T)> = None;

        loop {
            match node {
                Node::Leaf(ref leaf) => {
                    if let Some(col) = leaf.primitive.intersect(ray) {
                        closest_collision = match closest_collision {
                            Some((best, o)) =>  {
                                if col.distance < best.distance {
                                    Some((col, &self.items[leaf.obj]))
                                } else {
                                    Some((best, o))
                                }
                            },
                            None => Some((col, &self.items[leaf.obj])),
                        };
                    }
                    if stack_ptr == 0 {
                        break;
                    } else {
                        stack_ptr -= 1;
                        node = stack[stack_ptr].expect("Stack entry is not None");
                    }
                },
                Node::Cluster(clus) => {
                    let left_col = ray_box_collide(&ray, &clus.left.aabb(), closest_collision.map(|(best, _)| best.distance));
                    let right_col = ray_box_collide(&ray, &clus.right.aabb(), closest_collision.map(|(best, _)| best.distance));
                    match (left_col, right_col) {
                        (Some(ld), Some(rd)) => {
                            if ld < rd {
                                stack[stack_ptr] = Some(&clus.right);
                                stack_ptr += 1;
                                node = &clus.left;
                            } else {
                                stack[stack_ptr] = Some(&clus.left);
                                stack_ptr += 1;
                                node = &clus.right;
                            }
                        },
                        (Some(_), None) => node = &clus.left,
                        (None, Some(_)) => node = &clus.right,
                        (None, None) => if stack_ptr == 0 {
                            break;
                        } else {
                            stack_ptr -= 1;
                            node = stack[stack_ptr].expect("Stack entry is not None");
                        }
                    }
                },
            }
        }
        closest_collision
    }
}

// This algorithm for constructing the BVH taken from http://graphics.cs.cmu.edu/projects/aac/aac_build.pdf
// Note that the authors of this paper made several optimizations to get the reported construction speed.
// I'm omitting the optimizations for now and just implementing the base algorithm.
// Parameters:
// Delta is the traversal stopping threshold.  Naming this const DELTA to match the paper.
// Lower is faster, higher is better.  The paper suggests values between 4 and 20.
const DELTA: usize = 10;

// Cluster count reduction functrion f.  Here named ccrf for clarity.
// This ccrf taken from the paper.
const EPSILON: f64 = 0.01;
fn ccrf(x: usize) -> usize {
    let xf: f64 = x as f64;
    let c = (DELTA as f64).powf(0.5 - EPSILON) / 2.0;
    (c * xf.powf(0.5 - EPSILON)).ceil() as usize
}

pub fn construct_bvh_aac<T>(mut items_with_geometry: Vec<(Primitive, T)>) -> BVH<T> {
    let start_time = Instant::now();
    println!("[{:.2?}] Constructing BVH from {:?} objects", start_time.elapsed(), items_with_geometry.len());

    let mut nodes: Vec<Node> = items_with_geometry.iter()
        .enumerate()
        .map(|(ix, (p, _))| Node::Leaf(LeafNode::new(ix, *p)))
        .collect();
    let items: Vec<T> = items_with_geometry.drain(..).map(|(_, item)| item).collect();

    let num_bits = (nodes.len() as f64).log(4.0).ceil() as u16;
    if num_bits > 16 { panic!("Too many objects to construct BVH"); }

    println!("[{:.2?}] Performing morton code sort", start_time.elapsed());

    // Figure out how much we should scale by when computing morton codes.
    // Need to make sure that the largest bit of the largest component fits in num_bits.
    // But also want as much precision as possible.
    let cap = (1 << num_bits) as f64;
    let max = nodes.iter()
        .map(|n| n.aabb().center.max())
        .fold(0./0., f64::max);  // Hack to get max for floats.
    let scale = cap / max;

    let mut nodes_with_mc: Vec<(Node, u64)> = nodes.drain(..).map(|n| {
        let c = n.aabb().center;
        let mc = morton_code(num_bits, (c.x * scale) as u16, (c.y * scale) as u16, (c.z * scale) as u16);
        (n, mc)
    }).collect();

    // Sort by morton code.
    nodes_with_mc.sort_unstable_by_key(|(_, mc)| *mc);

    println!("[{:.2?}] Recursively constructing hierarchy", start_time.elapsed());

    let clusters: Vec<Node> = build_tree(nodes_with_mc, num_bits, 0);

    println!("[{:.2?}] Combining final clusters", start_time.elapsed());

    let mut final_clusters: Vec<Node> = combine_clusters(clusters, 1);

    let root = final_clusters.pop().expect("Must have at least one cluster");

    println!("[{:.2?}] Finished constructing BVH", start_time.elapsed());

    BVH { items, root }
}

fn build_tree(mut clusters: Vec<(Node, u64)>, max_depth: u16, depth: u16) -> Vec<Node> {
    let num_clusters = clusters.len();
    if num_clusters < DELTA {
        return combine_clusters(clusters.drain(..).map(|(n, _)| n).collect(), ccrf(DELTA));
    }

    let (lhs, rhs) = if depth < max_depth {
        make_partition(clusters, depth)
    } else {
        let mid = clusters.len() / 2;
        let rhs = clusters.split_off(mid);
        (clusters, rhs)
    };

    // Fork threads for first 2 layers.
    let new_clusters = if depth < 2 {
        let left_clusters_hdl = std::thread::spawn(move || build_tree(lhs, max_depth, depth + 1));
        let right_clusters_hdl = std::thread::spawn(move || build_tree(rhs, max_depth, depth + 1));
        let left_clusters = left_clusters_hdl.join().expect("Asynchronous task succeeded");
        let mut right_clusters = right_clusters_hdl.join().expect("Asynchronous task succeeded");
        let mut new_clusters = left_clusters;
        new_clusters.append(&mut right_clusters);
        new_clusters
    } else {
        let mut new_clusters = build_tree(lhs, max_depth, depth + 1);
        new_clusters.append(&mut build_tree(rhs, max_depth, depth + 1));
        new_clusters
    };
    
    combine_clusters(new_clusters, ccrf(num_clusters))
}

fn make_partition(mut clusters: Vec<(Node, u64)>, depth: u16) -> (Vec<(Node, u64)>, Vec<(Node, u64)>) {
    // Partition based on the current bit of the morton code.
    // Since the clusters are sorted, we can just binary search for where this bit changes from 0
    // to 1.

    // Handle edge cases first.
    if clusters.len() == 0 {
        return (vec![], vec![]);
    } else if get_bit(clusters.first().expect("Not empty").1, depth) {
        return (vec![], clusters);
    } else if !get_bit(clusters.last().expect("Not empty").1, depth) {
        return (clusters, vec![]);
    }

    let mut max_0: usize = 0;
    let mut min_1: usize = clusters.len() - 1;
    while min_1 - max_0 > 1 {
        let mid: usize = (min_1 + max_0) / 2;
        if get_bit(clusters[mid].1, depth) {
            min_1 = mid;
        } else {
            max_0 = mid;
        }
    }

    let rhs = clusters.split_off(min_1);
    (clusters, rhs)
}

fn combine_clusters(mut clusters: Vec<Node>, n: usize) -> Vec<Node> {
    // Lookup table from cluster index to index of "closest" cluster.
    let mut closest: Vec<usize> = Vec::with_capacity(clusters.len());

    for ix in 0 .. clusters.len() {
        closest.push(find_best_match(&clusters, ix));
    }

    while clusters.len() > n {
        // Find best pair to combine.
        let mut best = std::f64::MAX;
        let mut left: usize = 0;
        let mut right: usize = 0;
        for ix in 0 .. clusters.len() {
            let c = cost(&clusters[ix], &clusters[closest[ix]]);
            if c < best {
                best = c;
                left = ix;
                right = closest[ix];
            }
        }

        // Remove them from the current lists and add the combined cluster.
        if right < left {
            std::mem::swap(&mut right, &mut left);
        }
        let lc = clusters.remove(right);
        let rc = clusters.remove(left);
        closest.remove(right);
        closest.remove(left);

        let combined = Node::Cluster(ClusterNode::new(Box::new(lc), Box::new(rc)));
        clusters.push(combined);
        closest.push(find_best_match(&clusters, clusters.len() - 1));

        // Adjust or recompute any invalidated closest pairs.
        for ix in 0 .. clusters.len() {
            if closest[ix] == left || closest[ix] == right {
                closest[ix] = find_best_match(&clusters, ix);
            } else if closest[ix] >= right {
                closest[ix] -= 2;
            } else if closest[ix] >= left {
                closest[ix] -= 1;
            }
        }
    }

    clusters
}

fn find_best_match(clusters: &Vec<Node>, ix: usize) -> usize {
    let mut lowest_cost = std::f64::MAX;
    let mut best_jx: usize = 0;
    for jx in 0 .. clusters.len() {
        if ix == jx { continue; }

        let cix = &clusters[ix];
        let cjx = &clusters[jx];

        let c = cost(cix, cjx);
        if c < lowest_cost {
            lowest_cost = c;
            best_jx = jx;
        }
    }
    best_jx
}

// Cost is the surface area of the combined bounding box.
fn cost(c1: &Node, c2: &Node) -> f64 {
    let aabb1 = c1.aabb();
    let aabb2 = c2.aabb();
    let combined_aabb = combine_aabb(aabb1, aabb2);
    surface_area(combined_aabb)
}

fn combine_aabb(aabb1: &AABB, aabb2: &AABB) -> AABB {
    let min = Vector3::new(
        aabb1.min.x.min(aabb2.min.x),
        aabb1.min.y.min(aabb2.min.y),
        aabb1.min.z.min(aabb2.min.z),
        );

    let max = Vector3::new(
        aabb1.max.x.max(aabb2.max.x),
        aabb1.max.y.max(aabb2.max.y),
        aabb1.max.z.max(aabb2.max.z),
        );

    AABB::new(min, max)
}

fn surface_area(aabb: AABB) -> f64 {
    let w = aabb.max.x - aabb.min.x;
    let h = aabb.max.y - aabb.min.y;
    let d = aabb.max.z - aabb.min.z;
    2.0 * (w*h + h*d + d*w)
}

// Using u16s here so the final morton code will fit in a u64.
// This should still give us 16 bits of precision.
// The authors of the paper recommended using log4(N) bits, where N is the number of objects in the
// scene.
// 16 bits is enough to scale to many millions of triangles, so we should be good
fn morton_code(num_bits: u16, mut x: u16, mut y: u16, mut z: u16) -> u64 {
    let mut mc: u64 = 0;
    for ix in 0 .. num_bits {
        mc |= ((z & 1) as u64) << (64 - (num_bits * 3) + (ix * 3));
        mc |= ((y & 1) as u64) << (64 - (num_bits * 3) + (ix * 3) + 1);
        mc |= ((x & 1) as u64) << (64 - (num_bits * 3) + (ix * 3) + 2);
        x = x >> 1;
        y = y >> 1;
        z = z >> 1;
    }
    mc
}

fn get_bit(mc: u64, bit: u16) -> bool {
    ((mc >> (63 - bit)) & 1) == 1
}

#[cfg(test)]
mod test {
    use crate::bvh;

    #[test]
    fn test_morton_code() {
        let mc = bvh::morton_code(4, 0b0000_1001, 0b0000_1100, 0b0000_0011);
        assert_eq!(mc, 0b1100_1000_1101_0000__0000_0000_0000_0000__0000_0000_0000_0000__0000_0000_0000_0000);
    }
}
