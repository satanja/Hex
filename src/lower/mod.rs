//! Module to compute lower bounds for the graph
use crate::graph::{Graph, Undirected};
mod vc_rilp;
mod cycle_rilp;

pub fn lower_bound(graph: &Graph) -> usize {
    if graph.is_undirected() {
        vc_rilp::lower_bound(graph)
    } else {
        cycle_rilp::lower_bound(graph)
    }
}