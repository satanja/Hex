//! Module to compute lower bounds for the graph
use crate::graph::Graph;
mod cycle_rilp;
mod ecc_rilp;
mod vc_rilp;

pub fn lower_bound(graph: &Graph) -> usize {
    ecc_rilp::lower_bound(graph)
}
