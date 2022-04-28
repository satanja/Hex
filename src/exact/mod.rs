use crate::graph::{Graph, HeuristicReduce, Undirected};

mod adv_ilp;
mod backtracking;
mod cycle_ilp;
mod heur_ilp;
mod hybrid_ilp;
mod vc_ilp;
mod vc_solver;

pub fn solve(mut graph: Graph) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }

    if graph.is_undirected() {
        if let Some(mut reduced_solution) = vc_ilp::solve(&graph) {
            solution.append(&mut reduced_solution);
        }
    } else if let Some(mut reduced_solution) = adv_ilp::solve(&mut graph) {
        solution.append(&mut reduced_solution);
    }
    solution
}
