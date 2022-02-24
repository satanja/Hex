use std::process::{Command, Output, Stdio};

use crate::graph::Graph;
use crate::graph::Undirected;

fn extract_vc_solution(output: Output, solution: &mut Vec<u32>) {
    if output.status.success() {
        let string = std::str::from_utf8(&output.stdout).unwrap();

        let mut first = true;
        for result in string.lines() {
            if first {
                first = false;
                continue;
            }
            let vertex = result.parse::<u32>().unwrap() - 1;
            solution.push(vertex);
        }
    }
}

pub fn solve(graph: &Graph, solution: &mut Vec<u32>) {
    let mut child = Command::new("./extern/vc_solver")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.take() {
        graph.write_to_stdin(stdin);
    }

    let output = child.wait_with_output().unwrap();
    extract_vc_solution(output, solution);
}
