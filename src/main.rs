//! Main entry point of the contest-deliverable. A Graph is read from stdin and
//! is then supplied to the algorithm, and after, the solution is written to
//! stdout.

use heur::{Heuristic, SimulatedAnnealing};
mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;

fn main() {
    let graph = io::read().unwrap();
    let solution = SimulatedAnnealing::upper_bound(&graph);
    io::write(solution);
}
