use crate::{
    graph::{Graph, HeuristicReduce},
    io::Config,
};
use grb::prelude::*;

// mod backtracking;
// mod cycle_ilp;
// mod ilp;
mod splitter;
// mod heur_ilp;
// mod hybrid_ilp;
// mod reduce_ilp;
// mod vc_ilp;
mod vc_solver;
// mod vcsr_ilp;

mod grb_ilp;
pub fn solve(mut graph: Graph, config: &Config) -> Vec<u32> {
    let mut solution = graph.reduce();
    if graph.vertices() == 0 {
        return solution;
    }
    let mut remaining = grb_ilp::solve(graph, config);
    solution.append(&mut remaining);
    solution
}

pub fn init_model() -> Model {
    let mut env = Env::empty().unwrap();
    env.set(param::OutputFlag, 0).unwrap();
    env.set(param::Threads, 1).unwrap();
    let e = env.start().unwrap();
    let model = Model::with_env("ilp", e).unwrap();
    model
}

fn recover_solution(model: &Model, vars: &[Var], dfvs: &mut Vec<u32>) {
    dfvs.clear();
    for i in 0..vars.len() {
        let value = model.get_obj_attr(attr::X, &vars[i]).unwrap();
        if value >= 0.9995 {
            dfvs.push(i as u32);
        }
    }
}
