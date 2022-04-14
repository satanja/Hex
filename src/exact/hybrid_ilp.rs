use crate::graph::Graph;
use coin_cbc::{Col, Model, Sense, Solution};

pub fn solve(graph: &Graph) -> Option<Vec<u32>> {
    let mut model = Model::default();
    model.set_parameter("log", "0");
    
    // Optil can only use coinor-libcbc-dev version 2.8.12, which has
    // a bugged preprocessor
    #[cfg(feature = "cbc-old")]
    model.set_parameter("preprocess", "off");

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    let stars = graph.stars();
    for (u, list) in stars {
        for v in list {
            if u < v {
                let cstr = model.add_row();
                model.set_row_lower(cstr, 1.);
                model.set_weight(cstr, vars[u as usize], 1.);
                model.set_weight(cstr, vars[v as usize], 1.);
            }
        }
    }

    model.set_obj_sense(Sense::Minimize);
    let mut dfvs = Vec::new();
    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());

    loop {
        let mut changed = false;
        while let Some(cycle) = graph.find_cycle_with_fvs(&dfvs) {
            changed = true;
            dfvs.push(cycle[0]);
            let row = model.add_row();
            model.set_row_lower(row, 1.);
            for vertex in cycle {
                model.set_weight(row, vars[vertex as usize], 1.);
            }
        }

        if !changed {
            break;
        }

        let _out = shh::stdout();

        let solution = model.solve();

        recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    }
    Some(dfvs)
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
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn cycle_test_002() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.add_arc(4, 0);
        let solution = solve(&graph).unwrap();
        assert_eq!(solution.len(), 2);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
