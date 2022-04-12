//! Main entry point of the contest-deliverable. A Graph is read from stdin and
//! is then supplied to the algorithm, and after, the solution is written to
//! stdout.

mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;

fn main() {
    let graph = io::read().unwrap();
    let solution = exact::solve(graph);
    io::write(solution);
}
