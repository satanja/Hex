//! Vertex Cover ILP solver
use crate::graph::{EdgeIter, Graph};
use coin_cbc::{Model, Sense};

pub fn solve(graph: &Graph) -> Vec<u32> {
    let mut model = Model::default();
    model.set_parameter("log", "0");
    // TODO possible optimization flags

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    let edges = graph.undir_edge_iter();

    for (u, v) in edges {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);

        model.set_weight(cstr, vars[u as usize], 1.);
        model.set_weight(cstr, vars[v as usize], 1.);
    }

    model.set_obj_sense(Sense::Minimize);
    let solution = model.solve();

    (0..vars.len())
        .filter(|i| solution.col(vars[*i]) >= 0.95)
        .map(|i| i as u32)
        .collect()
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
    fn clique_test_001() {
        let n = 3;
        let graph = generate_clique(n);
        let solution = solve(&graph);
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn clique_test_002() {
        let n = 4;
        let graph = generate_clique(n);
        let solution = solve(&graph);
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }

    #[test]
    fn clique_test_003() {
        let n = 5;
        let graph = generate_clique(n);
        let solution = solve(&graph);
        assert_eq!(solution.len(), n - 1);
        assert!(graph.is_acyclic_with_fvs(&solution));
    }
}
