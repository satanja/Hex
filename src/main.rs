mod exact;
mod graph;
mod heur;
mod io;

use graph::Graph;
use io::{read, write};

use crate::exact::branch_and_bound;

fn main() {
    let mut graph = read().unwrap();
    let g2 = graph.clone();
    let upper_bound = graph.total_vertices() - heur::greedy(&graph).len(); // substitute for a better upper bound computation
    println!("{}", upper_bound);
    if let Some(solution) = branch_and_bound(&mut graph, &mut 0, upper_bound) {
        write(solution);
    }
}
