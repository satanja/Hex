//! Main file specifically for debugging: VScode does not support piping
//! with launch tasks, and manually supplying the file to the stdin is
//! cumbersome. Instead of reading from stdin, all instances in a directory are
//! loaded in and ran on whatever is in the main function (this will change
//! throughout the project).

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
use graph::Reducable;

fn main() {
    let paths = fs::read_dir("./instances/").unwrap();
    for path in paths {
        let pb = path.unwrap().path();
        let mut graph = io::read_from_path(pb).unwrap();
        heur::greedy_and_reduce(&graph);
    }
}
