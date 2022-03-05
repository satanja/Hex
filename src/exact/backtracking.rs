use crate::{
    graph::{Graph, Reducable},
    heur::{GRMaxDegree, Heuristic},
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
    if let Some((vertex, neighbors)) = graph.max_degree_star() {
        let mut best;

        let mut first = graph.clone();
        first.remove_vertex(vertex);
        best = branch_and_bound(&first, current_solution + 1, upper_bound);

        let mut second = graph.clone();
        second.remove_vertices(&neighbors);

        let b = branch_and_bound(&second, current_solution + neighbors.len(), upper_bound);
        best = min_solution(best, b);

        return best;
    }

    // we cannot branch on a star, so branch on a vertex in a cycle
    let cycle = graph.find_cycle_with_fvs(&vec![]).unwrap();
    let mut best = None;
    for vertex in cycle {
        let mut clone = graph.clone();
        clone.remove_vertex(vertex);

        let branch = branch_and_bound(&clone, current_solution + 1, upper_bound);
        best = min_solution(best, branch);
    }
    return best;
}

fn branch_and_reduce(graph: &mut Graph, upper_bound: usize) -> Option<Vec<u32>> {
    // first, reduce the graph as much as possible
    let reduced = graph.reduce(upper_bound);
    if !graph.is_cyclic() {
        // `reduced` is an optimal solution (even if empty)
        return Some(reduced);
    }

    let new_upper = upper_bound - reduced.len();
    let lb = lower::lower_bound(graph);
    if reduced.len() + lb > new_upper {
        return None;
    }

    // branch on the stars if possible
    if let Some((vertex, neighbors)) = graph.max_degree_star() {
        let mut best;

        let mut first = graph.clone();
        first.remove_vertex(vertex);
        best = branch_and_reduce(&mut first, new_upper - 1);

        let mut second = graph.clone();
        second.remove_vertices(&neighbors);

        let b = branch_and_reduce(&mut second, new_upper - neighbors.len());
        best = min_solution(best, b);

        return merge(best, reduced);
    }

    // we cannot branch on a star, so branch on a vertex in a cycle
    let cycle = graph.find_cycle_with_fvs(&vec![]).unwrap();
    let mut best = None;
    for vertex in cycle {
        let mut clone = graph.clone();
        clone.remove_vertex(vertex);

        let branch = branch_and_reduce(&mut clone, new_upper - 1);
        best = min_solution(best, branch);
    }

    merge(best, reduced)
}

pub fn solve(graph: &mut Graph) -> Vec<u32> {
    let ub = GRMaxDegree::upper_bound(graph);
    branch_and_reduce(graph, ub.len()).unwrap()
}

/// Returns the smallest solution between `a` and `b`. In case that `b` is `None`, return `a`.
/// In case `a` is `None` (and `b` is not `None`), return `b`.
fn min_solution(a: Option<Vec<u32>>, b: Option<Vec<u32>>) -> Option<Vec<u32>> {
    match (a.as_ref(), b.as_ref()) {
        (_, None) => a,
        (None, Some(_)) => b,
        (Some(list_a), Some(list_b)) => {
            if list_a.len() >= list_b.len() {
                b
            } else {
                a
            }
        }
    }
}

/// Combines
fn merge(best: Option<Vec<u32>>, mut reduced: Vec<u32>) -> Option<Vec<u32>> {
    match best {
        None => None,
        Some(mut solution) => {
            solution.append(&mut reduced);
            Some(solution)
        }
    }
}
