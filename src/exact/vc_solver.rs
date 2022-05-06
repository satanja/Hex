use duct::cmd;
use std::process::Output;
use std::time::Duration;

use crate::graph::Graph;
use crate::graph::Undirected;

fn extract_vc_solution_from_output(output: Output, solution: &mut Vec<u32>) {
    if output.status.success() {
        let string = std::str::from_utf8(&output.stdout).unwrap();

        let mut first = true;
        for result in string.lines() {
            if !first {
                let vertex = result.parse::<u32>().unwrap() - 1;
                solution.push(vertex);
            } else {
                first = false;
            }
        }
    }
}

fn extract_vc_solution_from_bytes(bytes: &[u8], solution: &mut Vec<u32>) {
    let mut str = std::str::from_utf8(bytes).unwrap();
    str = str.trim_end();
    let output: Vec<_> = str.split('\n').collect();
    for i in 1..output.len() {
        let vertex = output[i].parse::<u32>().unwrap() - 1;
        solution.push(vertex);
    }
}

fn run_solver(graph: &Graph, solution: &mut Vec<u32>, time_limit: Duration) -> bool {
    let command = cmd!("./extern/WeGotYouCovered/vc_solver")
        .stdin_bytes(graph.as_string())
        .stdout_capture();

    let child = command.start().unwrap();

    if let Ok(Some(output)) = child.try_wait() {
        let data = &output.stdout;
        extract_vc_solution_from_bytes(data, solution);
        return true;
    }

    std::thread::sleep(time_limit);

    match child.try_wait() {
        Ok(Some(output)) => {
            let data = &output.stdout;
            extract_vc_solution_from_bytes(data, solution);
            return true;
        }
        _ => {
            child.kill().unwrap();
            return false;
        }
    }
}

pub fn solve(graph: &Graph, solution: &mut Vec<u32>) -> bool {
    if run_solver(graph, solution, Duration::from_millis(500)) {
        return true;
    }

    if run_solver(graph, solution, Duration::from_secs(5)) {
        return true;
    }

    if run_solver(graph, solution, Duration::from_secs(10)) {
        return true;
    }

    if run_solver(graph, solution, Duration::from_secs(100)) {
        return true;
    }

    false
}
