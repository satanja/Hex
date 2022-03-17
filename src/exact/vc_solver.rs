use std::io::Read;
use std::process::{Command, Output, Stdio};
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
    let mut child = Command::new("./extern/WeGotYouCovered/vc_solver")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    if let Some(stdin) = child.stdin.take() {
        graph.write_to_stdin(stdin);
    }

    if let Ok(Some(_)) = child.try_wait() {
        let output = child.wait_with_output().unwrap();
        extract_vc_solution_from_output(output, solution);
        return true;
    }

    std::thread::sleep(time_limit);
    child.kill().unwrap();

    let mut stdout = child.stdout.unwrap();
    let mut buf = Vec::new();
    stdout.read_to_end(&mut buf).unwrap();

    if buf.len() == 0 {
        return false;
    }

    extract_vc_solution_from_bytes(&buf, solution);

    return true;
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

    return false;
}
