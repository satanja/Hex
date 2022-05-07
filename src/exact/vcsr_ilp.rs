use super::recover_solution;
use crate::{
    exact::vc_solver,
    graph::{EdgeCycleCover, Graph, Reducable, ThreeClique},
    util::reduce_hitting_set,
};
use coin_cbc::{Col, Model, Sense};
use rustc_hash::FxHashSet;
use std::fmt::Write;

pub fn solve(graph: &mut Graph) -> Option<Vec<u32>> {
    let vertices = graph.total_vertices();
    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();

    // Start form the undirected part of the graph
    // Include the undirected edges as constraints, and remove the undirected
    // edges from the graph. Safely reduce the graph (endpoints cannot be
    // reduced). Repeat until no undirected edges exist.
    loop {
        let stars = graph.stars();
        if stars.is_empty() {
            break;
        }

        let mut sources = Vec::with_capacity(stars.len());
        for (source, neighbors) in &stars {
            for neighbor in neighbors {
                if *source < *neighbor {
                    constraint_map[*source as usize].push(constraints.len());
                    constraint_map[*neighbor as usize].push(constraints.len());
                    constraints.push(vec![*source, *neighbor]);
                }
            }
            sources.push(*source);
        }
        // break;
        graph.mark_forbidden(&sources);
        graph.remove_undirected_edges(stars);

        let mut reduced = graph.reduce(vertices).unwrap();
        if reduced.is_empty() {
            break;
        }
        forced.append(&mut reduced);
    }

    // Some generated constraints may already be satisfied, filter those.
    let mut forced_constraints = FxHashSet::default();
    for vertex in &forced {
        for constraint in &constraint_map[*vertex as usize] {
            forced_constraints.insert(*constraint);
        }
    }
    let mut preprocess_constraints = Vec::new();
    for i in 0..constraints.len() {
        if forced_constraints.contains(&i) {
            continue;
        }
        preprocess_constraints.push(std::mem::take(&mut constraints[i]));
    }
    drop(constraints);

    let mut dfvs = Vec::new();
    if !preprocess_constraints.is_empty() {
        preprocess_constraints.sort_by(|a, b| a[0].cmp(&b[0]));
        let mut input = String::new();
        writeln!(
            input,
            "p td {} {}",
            graph.total_vertices(),
            preprocess_constraints.len()
        )
        .unwrap();
        for cstr in &preprocess_constraints {
            writeln!(input, "{} {}", cstr[0] + 1, cstr[1] + 1).unwrap();
        }
        dfvs = vc_solver::solve_from_string(input);

        if graph.is_acyclic_with_fvs(&dfvs) {
            dfvs.append(&mut forced);
            return Some(dfvs);
        }
    }

    let mut model = super::init_model();
    model.set_obj_sense(Sense::Minimize);

    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..vertices {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    for constraint in &preprocess_constraints {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in constraint {
            model.set_weight(cstr, vars[*vertex as usize], 1.);
        }
    }

    for (a, b, c) in graph.three_clique() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 2.);
        model.set_weight(cstr, vars[a as usize], 1.);
        model.set_weight(cstr, vars[b as usize], 1.);
        model.set_weight(cstr, vars[c as usize], 1.);
    }

    for vertex in &dfvs {
        model.set_col_initial_solution(vars[*vertex as usize], 1.);
    }

    let _out = shh::stdout();
    dfvs.clear();
    model.solve();

    for cycle in graph.edge_cycle_cover() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in cycle {
            model.set_weight(cstr, vars[vertex as usize], 1.);
        }
    }

    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, vertices);
    if graph.is_acyclic_with_fvs(&dfvs) {
        dfvs.append(&mut forced);
        return Some(dfvs);
    }

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
    dfvs.append(&mut forced);
    Some(dfvs)
}
