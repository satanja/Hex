use crate::graph::Graph;
use coin_cbc::{Col, Model, Sense, Solution};

pub fn solve(graph: &Graph, init_cycles: usize) -> Vec<u32> {
    let _out = shh::stdout();
    let mut model = Model::default();

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }
    model.set_obj_sense(Sense::Minimize);

    let mut dfvs = Vec::new();
    for _ in 0..init_cycles {
        match graph.find_cycle_with_fvs(&dfvs) {
            Some(cycle) => {
                dfvs.push(cycle[0]);
                let row = model.add_row();
                model.set_row_lower(row, 1.);
                for vertex in cycle {
                    model.set_weight(row, vars[vertex as usize], 1.);
                }
            }
            None => break,
        }
    }

    if graph.is_acyclic_with_fvs(&dfvs) {
        let solution = model.solve();
        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
        return dfvs;
    }

    while let Some(cycle) = graph.find_cycle_with_fvs(&dfvs) {
        let row = model.add_row();
        model.set_row_lower(row, 1.);
        for vertex in cycle {
            model.set_weight(row, vars[vertex as usize], 1.);
        }
        let solution = model.solve();
        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    }

    dfvs
}

fn recover_solution(solution: &Solution, vars: &Vec<Col>, dfvs: &mut Vec<u32>, vertices: usize) {
    dfvs.clear();
    for i in 0..vertices {
        if solution.col(vars[i]) >= 0.95 {
            dfvs.push(i as u32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_test_001() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 0);
        let solution = solve(&graph, 0);
        assert_eq!(solution.len(), 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn cycle_test_002() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 0);
        let solution = solve(&graph, 1);
        assert_eq!(solution.len(), 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn cycle_test_003() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.add_arc(4, 0);
        let solution = solve(&graph, 1);
        assert_eq!(solution.len(), 2);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
