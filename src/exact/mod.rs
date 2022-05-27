use crate::{
    graph::{Graph, HeuristicReduce},
    io::Config,
};
use coin_cbc::{Col, Model, Solution};

mod backtracking;
mod cycle_ilp;
mod heur_ilp;
mod hybrid_ilp;
mod reduce_ilp;
mod vc_ilp;
mod vc_solver;
mod vcsr_ilp;

pub fn solve(mut graph: Graph, config: &Config) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }
    let mut remaining = vcsr_ilp::solve(&mut graph, config);
    solution.append(&mut remaining);
    solution
}

pub fn init_model() -> Model {
    let mut model = Model::default();
    model.set_parameter("log", "0");
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
