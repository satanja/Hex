//! Module to compute lower bounds for the graph
use crate::graph::Graph;
// mod cycle_rilp;
// mod ecc_rilp;
// mod vc_rilp;
// mod vcsr_rilp;

pub fn lower_bound(graph: &Graph) -> usize {
    0
    // let rlb = raw_lower_bound(graph);
    // (rlb - 1e-5).ceil() as usize
}

pub fn raw_lower_bound(graph: &Graph) -> f64 {
    0.0
    // vcsr_rilp::lower_bound(graph)
}
