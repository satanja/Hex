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

use graph::{BFSSampler, Reducable};
use heur::{GRCycle, Heuristic};
use std::{fs, path::PathBuf, str::FromStr};

use crate::graph::Statistics;

fn main() {
    let paths = fs::read_dir("./instances/").unwrap();
    let mut file_names: Vec<_> = paths
        .into_iter()
        .map(|p| p.unwrap().path().display().to_string())
        .collect();
    file_names.sort();

    println!("dataset,vertices,edges,\"directed edges\",\"undirected edges\",\"avg degree\",\"compressed avg degree\",diameter,\"reduced diameter\",\"unreachable vertices\",\"stars\",\"avg star neighborhood\",\"undirected components\",sccs");
    for path in file_names {
        let pb = PathBuf::from_str(&path).unwrap();

        let mut graph = io::read_from_path(&pb).unwrap();
        graph.reduce(graph.total_vertices());
        let vertices = graph.vertices();
        let edges = graph.edges();
        let dir_edges = graph.directed_edges();
        let undir_edges = graph.undirected_edges();
        let avg_degree = graph.avg_degree();
        let cavg_degree = graph.compressed_avg_degree();
        let diameter = graph.diameter();
        let rdiameter = graph.reduced_diameter();
        let unreachable = graph.unreachable_vertices();
        let stars = graph.number_of_stars();
        let avg_star_ngh = graph.avg_star_neighborhood();
        let undir_components = graph.number_of_undirected_components();
        let sccs = graph.strongly_connected_components();
        println!(
            "{},{},{},{},{},{:.2},{:.2},{},{},{},{},{:.2},{},{}",
            pb.file_name().unwrap().to_str().unwrap(),
            vertices,
            edges,
            dir_edges,
            undir_edges,
            avg_degree,
            cavg_degree,
            diameter,
            rdiameter,
            unreachable,
            stars,
            avg_star_ngh,
            undir_components,
            sccs
        );
    }
}
