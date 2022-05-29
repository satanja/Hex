use crate::graph::Graph;
// use clap::Parser;
use io::Result;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

pub fn read() -> Result<Graph> {
    let mut line = String::new();
    let stdin = io::stdin();

    stdin.read_line(&mut line)?;
    while line.starts_with('%') {
        line.clear();
        stdin.read_line(&mut line)?;
    }

    let specs: Vec<_> = line
        .clone()
        .trim_end()
        .split_whitespace()
        .map(|v| v.parse::<usize>().unwrap())
        .collect();
    line.clear();

    let vertices = specs[0];

    let mut graph = Graph::new(vertices);

    let mut index = 0;
    while index < vertices {
        stdin.read_line(&mut line)?;
        while line.starts_with('%') {
            line.clear();
            stdin.read_line(&mut line)?;
        }
        let adj: Vec<u32> = line
            .clone()
            .trim_end()
            .split_whitespace()
            .map(|v| v.parse::<u32>().unwrap() - 1)
            .collect();
        graph.set_adjacency(index as u32, adj);

        line.clear();
        index += 1;
    }
    Ok(graph)
}

pub fn read_from_path(path: &PathBuf) -> Result<Graph> {
    let mut line = String::new();
    let file = File::open(path).unwrap();
    let mut reader = BufReader::new(file);

    reader.read_line(&mut line)?;
    while line.starts_with('%') {
        line.clear();
        reader.read_line(&mut line)?;
    }

    let specs: Vec<_> = line
        .clone()
        .trim_end()
        .split_whitespace()
        .map(|v| v.parse::<usize>().unwrap())
        .collect();
    line.clear();

    let vertices = specs[0];

    let mut graph = Graph::new(vertices);

    let mut index = 0;
    while index < vertices {
        reader.read_line(&mut line)?;
        while line.starts_with('%') {
            line.clear();
            reader.read_line(&mut line)?;
        }
        let adj: Vec<u32> = line
            .clone()
            .trim_end()
            .split_whitespace()
            .map(|v| v.parse::<u32>().unwrap() - 1)
            .collect();
        graph.set_adjacency(index as u32, adj);

        line.clear();
        index += 1;
    }

    // graph.initialize_data_structures();
    Ok(graph)
}

pub fn write(solution: Vec<u32>) {
    for vertex in solution {
        println!("{}", vertex + 1);
    }
}

// #[derive(Parser, Debug)]
// #[clap(author, version, about, long_about = None)]
pub struct Config {
    /// WeGotYouCovered's vertex cover solver time limit (s)
    // #[clap(short, long, default_value_t = 300)]
    time_limit_vc: u64,
}

impl Config {
    pub fn time_limit_vc(&self) -> u64 {
        self.time_limit_vc
    }
}

pub fn config() -> Config {
    Config { time_limit_vc: 300 }
}
