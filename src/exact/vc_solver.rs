use assert_cmd::Command;
use std::process::Output;
use std::time::Duration;

use crate::graph::{Graph, Undirected};

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

fn run_solver(graph: &Graph, solution: &mut Vec<u32>, time_limit: Option<Duration>) -> bool {
    let mut program = create_program_launch();
    let command = if let Some(duration) = time_limit {
        program.write_stdin(graph.as_string()).timeout(duration)
    } else {
        program.write_stdin(graph.as_string())
    };

    let stdout;
    match command.ok() {
        Ok(output) => stdout = output.stdout,
        Err(_) => return false,
    }
    extract_vc_solution_from_bytes(&stdout, solution);
    true
}

pub fn solve(graph: &Graph, solution: &mut Vec<u32>) -> bool {
    if cfg!(feature = "time-limit") {
        if run_solver(graph, solution, Some(Duration::from_secs(8 * 60))) {
            return true;
        }
        return false;
    } else if cfg!(feature = "bench") {
        // rayon cannot clean up vc_solver on its own, so we must kill it ourselves
        // we clean it up after a little over 28 minutes have passed, all proceeding 
        // steps are always running within 2 minutes. 
        // this is very hacky, but it works...
        return run_solver(graph, solution, Some(Duration::from_secs(1700)));
    } else {
        return run_solver(graph, solution, None);
    }
}

fn create_program_launch() -> Command {
    if cfg!(feature = "root-vc-solver") {
        Command::new("./vc_solver")
    } else {
        Command::new("./extern/WeGotYouCovered/vc_solver")
    }
}
