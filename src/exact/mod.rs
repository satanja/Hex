use crate::graph::{Graph, HeuristicReduce};

mod backtracking;
mod cycle_ilp;
mod heur_ilp;
mod hybrid_ilp;
mod reduce_ilp;
mod vc_ilp;
mod vc_solver;

pub fn solve(mut graph: Graph) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }

    let mut back = reduce_ilp::solve(&mut graph).unwrap();
    // if graph.is_undirected() {
    //     if let Some(mut reduced_solution) = vc_ilp::solve(&graph) {
    //         solution.append(&mut reduced_solution);
    //     }
    // } else {
    //     if let Some(mut reduced_solution) = reduce_ilp::solve(&mut graph) {
    //         solution.append(&mut reduced_solution);
    //     }
    // }
    solution.append(&mut back);
    solution
}
