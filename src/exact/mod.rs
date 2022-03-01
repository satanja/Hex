use crate::graph::{Graph, HeuristicReduce, Undirected};
use crate::lower;

mod cycle_ilp;
mod vc_ilp;
mod vcsolver;

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

pub fn solve(mut graph: Graph) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }

    if graph.is_undirected() {
        println!("undirected");
        let mut reduced_solution = vc_ilp::solve(&graph);
        solution.append(&mut reduced_solution);
    } else {
        println!("directed");
        let mut reduced_solution = cycle_ilp::solve(&graph, 10);
        solution.append(&mut reduced_solution);
    }

    solution
}
