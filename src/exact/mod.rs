use crate::graph::{Graph, HeuristicReduce, Undirected};

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
        // println!("undirected");
        // vc_solver::solve(&graph, &mut solution);
        if let Some(mut reduced_solution) = vc_ilp::solve(&graph) {
            solution.append(&mut reduced_solution);
        } else {
            println!("time limit exceeded");
        }
    } else {
        // return Vec::new();
        if let Some(mut reduced_solution) = hybrid_ilp::solve(&graph) {
            solution.append(&mut reduced_solution);
        } else {
            println!("time limit exceeded");
        }
    }

    solution
}
