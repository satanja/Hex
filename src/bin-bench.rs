//! Main file to benchmark the solver
mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;

use std::{fs, path::PathBuf, str::FromStr};
use std::time::Instant;

fn main() {
    let paths = fs::read_dir("./instances/").unwrap();
    let mut file_names: Vec<_> = paths
        .into_iter()
        .map(|p| p.unwrap().path().display().to_string())
        .collect();
    file_names.sort();

    for path in file_names {
        println!("{:?}", path);
        let pb = PathBuf::from_str(&path).unwrap();
        let graph = io::read_from_path(&pb).unwrap();

        let start = Instant::now();
        let solution = exact::solve(graph);
        let end = Instant::now();

        println!("{}", solution.len());
        println!("{:?}", end.duration_since(start));
    }
}
