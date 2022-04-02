//! Main file specifically for debugging: VScode does not support piping
//! with launch tasks, and manually supplying the file to the stdin is
//! cumbersome. Instead of reading from stdin, all instances in a directory are
//! loaded in and ran on whatever is in the main function (this will change
//! throughout the project).

mod exact;
mod graph;
mod heur;
mod io;
mod lower;
mod util;


use heur::{Heuristic, SimulatedAnnealing};
use std::{fs, path::PathBuf, str::FromStr};



fn main() {
    let paths = fs::read_dir("./instances/").unwrap();
    let mut file_names: Vec<_> = paths
        .into_iter()
        .map(|p| p.unwrap().path().display().to_string())
        .collect();
    file_names.sort();

    for path in file_names {
        let pb = PathBuf::from_str(&path).unwrap();

        let graph = io::read_from_path(&pb).unwrap();
        let upper_bound = SimulatedAnnealing::upper_bound(&graph);
        println!("{}", upper_bound.len());
    }
}
