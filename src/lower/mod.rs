//! Module to compute lower bounds for the graph
use crate::graph::Graph;
mod vc_rilp;

pub fn lower_bound(graph: &Graph) -> usize {
    vc_rilp::lower_bound(graph)
}