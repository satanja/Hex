use crate::graph::Graph;
use std::collections::HashSet;

pub fn greedy(graph: &Graph) -> Vec<u32> {
    let mut copy = graph.clone();
    while copy.is_cyclic() {
        let v = copy.max_degree_vertex();
        copy.disable_vertex(v);
    }
    return simple_post_processing(&mut copy);
}

pub fn simple_post_processing(graph: &mut Graph) -> Vec<u32> {
    let mut disabled_set: HashSet<_> = graph.get_disabled_vertices().into_iter().collect();
    loop {
        let mut to_delete = Vec::new();
        for vertex in &disabled_set {
            graph.enable_vertex_post(*vertex as u32);
            if graph.is_cyclic() {
                graph.disable_vertex_post(*vertex as u32);
            } else {
                to_delete.push(*vertex);
            }
        }

        if to_delete.len() == 0 {
            break;
        }

        for vertex in to_delete {
            disabled_set.remove(&vertex);
        }
    }
    graph.get_active_vertices()
}

pub fn greedy_and_reduce(graph: &mut Graph) -> Vec<u32> {
    greedy(graph)
}
