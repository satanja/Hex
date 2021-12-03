use crate::graph::Graph;

pub fn branch_and_bound(
    graph: &mut Graph,
    current_solution: &mut usize,
    upper_bound: usize,
) -> Vec<u32> {
    if !graph.has_cycle() {
        return Vec::new();
    }

    let vertices = graph.get_active_vertices();
    let mut best_size = upper_bound;
    let mut result = Vec::new();

    for vertex in vertices {
        graph.disable_vertex(vertex as u32);
        // current_solution + 1 for a better lower bound computation
        if *current_solution + 1 >= upper_bound {
            continue;
        }

        *current_solution += 1;
        let solution = branch_and_bound(graph, current_solution, best_size);

        if solution.len() + 1 < best_size {
            best_size = solution.len();
            result = solution;
            result.push(vertex as u32);
        }

        graph.enable_vertex(vertex as u32);
        *current_solution -= 1;
    }

    result
}
