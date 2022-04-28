use crate::{
    graph::{Graph, Reducable},
    heur::{Heuristic, SimulatedAnnealing},
    lower,
};

pub fn branch_and_bound(
    graph: &Graph,
    current_solution: usize,
    upper_bound: usize,
) -> Option<Vec<u32>> {
    if !graph.is_cyclic() {
        return Some(Vec::new());
    }

    let lb = lower::lower_bound(graph);
    if current_solution + lb > upper_bound {
        return None;
    }

    // branch on the stars if possible
    if let Some((vertex, mut neighbors)) = graph.max_degree_star() {
        let mut first = graph.clone();
        first.remove_vertex(vertex);
        let a = branch_and_bound(&first, current_solution + 1, upper_bound);

        let mut second = graph.clone();
        second.remove_vertices(&neighbors);
        let b = branch_and_bound(&second, current_solution + neighbors.len(), upper_bound);

        // determine the smallest solution and construct the new solution
        match min_solution(&a, &b, 1, neighbors.len()) {
            None => return None,
            Some(true) => {
                let mut solution = a.unwrap();
                solution.push(vertex);
                return Some(solution);
            }
            Some(false) => {
                let mut solution = b.unwrap();
                solution.append(&mut neighbors);
                return Some(solution);
            }
        }
    }

    // we cannot branch on a star, so branch on a vertex in a cycle
    let cycle = graph.find_cycle_with_fvs(&vec![]).unwrap();
    let mut best = None;
    for vertex in cycle {
        let mut clone = graph.clone();
        clone.remove_vertex(vertex);

        let branch = branch_and_bound(&clone, current_solution + 1, upper_bound);
        match min_solution(&best, &branch, 1, 1) {
            None => return None,
            Some(true) => {
                let mut solution = best.unwrap();
                solution.push(vertex);
                best = Some(solution);
            }
            Some(false) => {
                let mut solution = branch.unwrap();
                solution.push(vertex);
                best = Some(solution);
            }
        }
    }
    best
}

fn branch_and_reduce(graph: &mut Graph, upper_bound: usize) -> Option<Vec<u32>> {
    // first, reduce the graph as much as possible
    let reduce_op = graph.reduce(upper_bound);
    if reduce_op == None {
        // println!("killed: reduction exceeds upper bound");
        return None;
    }
    let mut reduced = reduce_op.unwrap();

    if !graph.is_cyclic() {
        // println!("found solution");
        return Some(reduced);
    }

    let mut new_upper = upper_bound - reduced.len();
    // let lb = lower::lower_bound(graph);
    // if lb > new_upper {
    //     // println!("shit");
    //     return None;
    // }

    // branch on the stars if possible
    if let Some((vertex, mut neighbors)) = graph.max_degree_star() {
        let mut first = graph.clone();
        first.remove_vertex(vertex);
        let a = branch_and_reduce(&mut first, new_upper - 1);

        if let Some(list) = a.as_ref() {
            // the reduced graph can be made acyclic by removing `vertex` and all vertices `list`
            // set the upper bound to `list.len()` to only find smaller solutions if they exist
            new_upper = list.len();
        }

        let b = if neighbors.len() > new_upper {
            None
        } else {
            let mut second = graph.clone();
            second.remove_vertices(&neighbors);
            branch_and_reduce(&mut second, new_upper - neighbors.len())
        };

        match min_solution(&a, &b, 1, neighbors.len()) {
            None => return None,
            Some(true) => {
                let mut solution = a.unwrap();
                solution.push(vertex);
                solution.append(&mut reduced);
                // println!("a: {}", solution.len());
                return Some(solution);
            }
            Some(false) => {
                let mut solution = b.unwrap();
                solution.append(&mut neighbors);
                solution.append(&mut reduced);
                // println!("b: {}", solution.len());
                return Some(solution);
            }
        }
    }

    // we cannot branch on a star, so branch on a vertex in a cycle
    let cycle = graph.find_cycle_with_fvs(&vec![]).unwrap();
    let mut best = None;
    for vertex in cycle {
        let mut clone = graph.clone();
        clone.remove_vertex(vertex);

        let branch = branch_and_reduce(&mut clone, new_upper - 1);
        match min_solution(&best, &branch, 1, 1) {
            None => return None,
            Some(true) => {
                let mut solution = best.unwrap();
                solution.push(vertex);
                best = Some(solution);
            }
            Some(false) => {
                let mut solution = branch.unwrap();
                solution.push(vertex);
                best = Some(solution);
            }
        }
    }
    let mut solution = best.unwrap();
    solution.append(&mut reduced);
    Some(solution)
}

pub fn solve(graph: &mut Graph) -> Vec<u32> {
    let ub = SimulatedAnnealing::upper_bound(graph);
    let mut solution = graph.reduce(ub.len()).unwrap();
    let components = graph.tarjan(true).unwrap();
    for component in components {
        let mut subgraph = graph.induced_subgraph(component);
        let ub = SimulatedAnnealing::upper_bound(&subgraph);
        let mut sub_solution = branch_and_reduce(&mut subgraph, ub.len()).unwrap();
        solution.append(&mut sub_solution);
    }
    solution
}

/// Returns whether `a` is a better solution than `b`, given that `delta_a` and `delta_b` vertices still
/// need to be included in `a` or `b` respectively. Returns `None` when both `a` and `b` are `None`.
fn min_solution(
    a: &Option<Vec<u32>>,
    b: &Option<Vec<u32>>,
    delta_a: usize,
    delta_b: usize,
) -> Option<bool> {
    match (a.as_ref(), b.as_ref()) {
        (None, None) => None,
        (None, Some(_)) => Some(false),
        (Some(_), None) => Some(true),
        (Some(list_a), Some(list_b)) => {
            let ans = list_a.len() + delta_a <= list_b.len() + delta_b;
            Some(ans)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_clique(vertices: usize) -> Graph {
        let mut graph = Graph::new(vertices);
        for i in 0..vertices {
            for j in i + 1..vertices {
                graph.add_arc(i as u32, j as u32);
                graph.add_arc(j as u32, i as u32);
            }
        }
        graph
    }

    #[test]
    fn branch_and_bound_test_001() {
        let n = 3;
        let graph = generate_clique(n);
        let solution = branch_and_bound(&graph, 0, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_bound_test_002() {
        let n = 4;
        let graph = generate_clique(n);
        let solution = branch_and_bound(&graph, 0, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_bound_test_003() {
        let n = 5;
        let graph = generate_clique(n);
        let solution = branch_and_bound(&graph, 0, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_reduce_test_001() {
        let n = 3;
        let mut graph = generate_clique(n);
        let solution = branch_and_reduce(&mut graph, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_reduce_test_002() {
        let n = 4;
        let mut graph = generate_clique(n);
        let solution = branch_and_reduce(&mut graph, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_reduce_test_003() {
        let n = 5;
        let mut graph = generate_clique(n);
        let solution = branch_and_reduce(&mut graph, n).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn branch_and_bound_test_004() {
        let n = 5;
        let mut graph = generate_clique(n);
        let ub = SimulatedAnnealing::upper_bound(&graph);
        let solution = branch_and_bound(&mut graph, 0, ub.len()).unwrap();
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
