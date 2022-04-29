use crate::graph::{Graph, Reducable, Compressor, EdgeCycleCover};
use coin_cbc::{Col, Model, Sense, Solution};
use crate::heur::{SimulatedAnnealing, Heuristic};
use rustc_hash::FxHashSet;

pub fn solve(graph: &mut Graph) -> Option<Vec<u32>> {
    let mut model = Model::default();
    model.set_parameter("log", "0");
    // let _out = shh::stdout();
    // let upper_bound = SimulatedAnnealing::upper_bound(&graph);

    let vertices = graph.total_vertices();
    let mut vars = Vec::with_capacity(vertices);
    for _ in 0..graph.total_vertices() {
        let var = model.add_binary();
        model.set_obj_coeff(var, 1.);
        vars.push(var);
    }

    // for vertex in upper_bound {
    //     model.set_col_initial_solution(vars[vertex as usize], 1.);
    // }

    let mut constraints = Vec::new();
    let mut constraint_map = vec![Vec::new(); vertices];
    let mut forced = Vec::new();
    
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
                    constraints.push([*source, *neighbor]);
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
    let mut forced_constraints = FxHashSet::default();
    
    for vertex in &forced {
        for constraint in &constraint_map[*vertex as usize] {
            forced_constraints.insert(*constraint);
        }
    }

    for i in 0..constraints.len() {
        if forced_constraints.contains(&i) {
           continue; 
        }

        let list = constraints[i];
        let u = list[0];
        let v = list[1];

        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        model.set_weight(cstr, vars[u as usize], 1.);
        model.set_weight(cstr, vars[v as usize], 1.);
    }

    let mut dfvs = Vec::new();
    model.set_obj_sense(Sense::Minimize);
    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    
    if graph.is_acyclic_with_fvs(&dfvs) {
        dfvs.append(&mut forced);
        return Some(dfvs);
    }
    
    for cycle in graph.edge_cycle_cover() {
        let cstr = model.add_row();
        model.set_row_lower(cstr, 1.);
        for vertex in cycle {
            model.set_weight(cstr, vars[vertex as usize], 1.);
        }
    }


    let solution = model.solve();
    recover_solution(&solution, &vars, &mut dfvs, graph.total_vertices());
    
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

fn recover_solution(solution: &Solution, vars: &Vec<Col>, dfvs: &mut Vec<u32>, vertices: usize) {
    dfvs.clear();
    for i in 0..vertices {
        if solution.col(vars[i]) >= 0.95 {
            dfvs.push(i as u32);
        }
    }
}