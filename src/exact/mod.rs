use crate::graph::{Graph, HeuristicReduce, Undirected};
use crate::heur::{GRMaxDegree, Heuristic};
use crate::lower;

mod backtracking;
mod cycle_ilp;
mod vc_ilp;
mod vc_solver;

pub fn solve(mut graph: Graph) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }

    if graph.is_undirected() {
        vc_solver::solve(&graph, &mut solution);
        // solution.append(&mut reduced_solution);
    } else {
        return Vec::new();
        // let mut reduced_solution = cycle_ilp::solve(&mut graph);
        // solution.append(&mut reduced_solution);
    }

    solution
}
