mod exact;
mod graph;
mod io;

use graph::Graph;
use io::{read, write};

use crate::exact::branch_and_bound;

fn main() {
    let mut graph = read().unwrap();
    let upper_bound = graph.total_vertices(); // substitute for a better upper bound computation
    let solution = branch_and_bound(&mut graph, &mut 0, upper_bound).unwrap();
    write(solution);
}
