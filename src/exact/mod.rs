use crate::graph::{Graph, HeuristicReduce};
use coin_cbc::{Col, Model, Solution};

mod backtracking;
mod cycle_ilp;
mod heur_ilp;
mod hybrid_ilp;
mod reduce_ilp;
mod vc_ilp;
mod vc_solver;
mod vcsr_ilp;

pub fn solve(mut graph: Graph) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }

    let mut back = vcsr_ilp::solve(&mut graph).unwrap();
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

fn init_model() -> Model {
    #[allow(unused_mut)]
    let mut model = Model::default();

    // Disable the bugged preprocessor for cbc 2.8.12
    // Optil servers use 2.8.12...
    #[cfg(feature = "old-cbc")]
    model.set_parameter("preprocess", "off");

    model
}

fn recover_solution(solution: &Solution, vars: &[Col], dfvs: &mut Vec<u32>, vertices: usize) {
    dfvs.clear();
    for i in 0..vertices {
        if solution.col(vars[i]) >= 0.9995 {
            dfvs.push(i as u32);
        }
    }
}
