mod exact;
mod graph;
mod heur;
mod io;
mod util;

use std::env;
use std::fs;

use graph::Graph;
use io::{read, write};

use crate::exact::branch_and_bound;

fn main() {
    // let args: Vec<_> = env::args().collect();
    let paths = fs::read_dir("./exact_instances/").unwrap();
    for path in paths {
        let pb = path.unwrap().path();
        let graph = io::read_from_path(pb).unwrap();
        println!("{}", graph.total_vertices() - heur::greedy(&graph).len());
    }

    // let mut graph = read().unwrap();
    // let g2 = graph.clone();
    // let upper_bound = graph.total_vertices() - heur::greedy(&graph).len(); // substitute for a better upper bound computation
    // println!("{}", upper_bound);
    // if let Some(solution) = branch_and_bound(&mut graph, &mut 0, upper_bound) {
    //     write(solution);
    // }
}
