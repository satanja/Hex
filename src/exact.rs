use crate::graph::Graph;

pub fn branch_and_bound(
    graph: &mut Graph,
    current_solution: &mut usize,
    at_least: u32,
    upper_bound: usize,
) -> Option<Vec<u32>> {
    // if !graph.is_cyclic() {
    //     return Some(Vec::new());
    // }

    // let vertices = graph.get_active_vertices();
    // let mut best_size = upper_bound;
    // let mut result = Vec::new();
    // let mut modified = false;

    // for vertex in vertices.iter().rev() {
    //     if *vertex < at_least {
    //         continue;
    //     }

    //     graph.disable_vertex(*vertex as u32);
    //     // current_solution + 1 for a better lower bound computation
    //     *current_solution += 1;
    //     // check whether we can recurse
    //     if *current_solution < upper_bound {
    //         if let Some(solution) = branch_and_bound(graph, current_solution, *vertex, best_size) {
    //             if solution.len() + 1 < best_size {
    //                 result = solution;
    //                 result.push(*vertex as u32);
    //                 best_size = result.len();
    //                 println!("{}", best_size);
    //                 modified = true;
    //             }
    //         }
    //     }

    //     *current_solution -= 1;
    //     graph.enable_vertex(*vertex as u32);
    // }

    // if modified {
    //     Some(result)
    // } else {
    //     None
    // }
    None
}
