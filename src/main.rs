//! Main entry point of the contest-deliverable. A Graph is read from stdin and
//! is then supplied to the algorithm, and after, the solution is written to
//! stdout.
mod exact;
mod graph;
mod heur;
mod io;
mod util;

use graph::Reducable;

fn main() {
    let mut graph = io::read().unwrap();
    graph.reduce(graph.total_vertices());
}
