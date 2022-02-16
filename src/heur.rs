use crate::{
    graph::{Graph, Reducable},
    util::RangeSet,
};
use std::collections::HashSet;

pub fn greedy(graph: &Graph) -> Vec<u32> {
    let mut copy = graph.clone();
    let mut solution = Vec::new();
    while copy.is_cyclic() {
        let v = copy.max_degree_vertex();
        copy.remove_vertex(v);
        solution.push(v);
    }
    make_minimal(&mut graph.clone(), solution)
}

/// Reduces the solution to a minimal solution. Tries to reintroduce a vertex to
/// the graph, and if the graph is still acyclic, we can continue. Otherwise,
/// that vertex must be removed from the graph.
pub fn make_minimal(graph: &mut Graph, solution: Vec<u32>) -> Vec<u32> {
    let mut set: HashSet<_> = solution.iter().map(|v| *v).collect();
    for vertex in &solution {
        graph.disable_vertex_post(*vertex);
    }

    for vertex in solution {
        graph.enable_vertex_post(vertex);
        if graph.is_cyclic() {
            graph.disable_vertex_post(vertex);
        } else {
            set.remove(&vertex);
        }
    }
    set.into_iter().collect()
}

pub fn greedy_and_reduce(graph: &Graph) -> Vec<u32> {
    let mut copy = graph.clone();
    let mut solution = Vec::new();
    solution.append(&mut copy.reduce());

    while copy.is_cyclic() {
        let v = copy.max_degree_vertex();
        copy.remove_vertex(v);
        solution.push(v);
        solution.append(&mut copy.reduce());
    }
    
    let p = make_minimal(&mut graph.clone(), solution);
    debug_assert!(!graph.has_cycle_with_fvs(&p));
    p
}
