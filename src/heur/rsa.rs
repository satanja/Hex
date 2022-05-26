use rustc_hash::FxHashSet;

use crate::{
    graph::{EdgeCycleCover, Graph, Reducable},
    util::Constraint,
};

use super::{ilp_upper_bound, Heuristic};

pub struct RSA {}
impl Heuristic for RSA {
    fn upper_bound(original: &Graph) -> Vec<u32> {
        let mut graph = original.clone();
        let vertices = graph.total_vertices();
        // let initial = Vec::new();
        let mut initial = graph.reduce(vertices).unwrap();

        if graph.is_empty() {
            return initial;
        }

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
                        constraints.push(Constraint::new(vec![*source, *neighbor], 1));
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
            for constraint_index in &constraint_map[*vertex as usize] {
                forced_constraints.insert(*constraint_index);
            }
        }
        let mut hitting_set = Vec::new();
        for i in 0..constraints.len() {
            if forced_constraints.contains(&i) {
                continue;
            }
            hitting_set.push(std::mem::take(&mut constraints[i]));
        }

        for cycle in graph.edge_cycle_cover() {
            hitting_set.push(Constraint::new(cycle, 1));
        }

        let mut upper_bound = ilp_upper_bound(&hitting_set, vertices);
        if graph.is_acyclic_with_fvs(&upper_bound) {
            upper_bound.append(&mut forced);
            upper_bound.append(&mut initial);
            return upper_bound;
        }

        loop {
            let mut changed = false;
            while let Some(cycle) = graph.find_cycle_with_fvs(&upper_bound) {
                changed = true;
                upper_bound.push(cycle[0]);
                hitting_set.push(Constraint::new(cycle, 1));
            }

            if !changed {
                break;
            }

            upper_bound = ilp_upper_bound(&hitting_set, vertices);
        }
        upper_bound.append(&mut forced);
        upper_bound.append(&mut initial);
        upper_bound
    }
}
