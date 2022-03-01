//! Vertex Cover based lower bound computation
use crate::graph::{EdgeIter, Graph};
use coin_cbc::{Model, Sense};

pub fn lower_bound(graph: &Graph) -> usize {
    let mut model = Model::default();
    model.set_parameter("log", "0");

    let mut vars = Vec::with_capacity(graph.total_vertices());
    for _ in 0..graph.total_vertices() {
        let var = model.add_col();
        model.set_col_lower(var, 0.);
        model.set_col_upper(var, 1.);
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
    solution.raw().obj_value().floor() as usize
}
