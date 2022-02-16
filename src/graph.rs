use core::fmt;
use std::{iter::Filter, usize, collections::BTreeMap, slice::SliceIndex};

use crate::util::{Heap, KeyValue, MaxItem, MinItem, RangeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    // /// The list of vertices which are not deleted.
    active_vertices: Vec<bool>,

    // ///
    num_active_vertices: usize,

    coloring: Vec<Color>,

    /// The adjacency list representation of the graph.
    adj: Vec<Vec<u32>>,

    /// The adjacency list representation of the reversed graph.
    rev_adj: Vec<Vec<u32>>,

    // /// Out-degree heap
    max_degree_heap: Heap<MaxItem>,

    // /// Current out-degree of each vertex
    current_out_degree: Vec<usize>,

    // /// Current in-degree of each vertex
    current_in_degree: Vec<usize>,

    // /// H
    sink_source_buffer: Vec<u32>,

    sinks_or_sources: RangeSet,

    // test
    deleted_vertices: Vec<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Color {
    Unvisited,
    Visited,
    Exhausted,
}

impl Graph {
    pub fn new(vertices: usize) -> Graph {
        let active_vertices = vec![true; vertices];
        let num_active_vertices = vertices;
        let coloring = vec![Color::Unvisited; vertices];
        let adj = vec![Vec::new(); vertices];
        let rev_adj = vec![Vec::new(); vertices];
        Graph {
            active_vertices,
            num_active_vertices,
            coloring,
            adj,
            rev_adj,
            max_degree_heap: Heap::new(0),
            current_out_degree: vec![0; vertices],
            current_in_degree: vec![0; vertices],
            sink_source_buffer: Vec::new(),
            sinks_or_sources: RangeSet::new(vertices),
            deleted_vertices: vec![false; vertices],
        }
    }

    pub fn add_arc(&mut self, source: u32, target: u32) {
        if let Err(index) = self.adj[source as usize].binary_search(&target) {
            self.adj[source as usize].insert(index, target);
        }
        if let Err(index) = self.rev_adj[target as usize].binary_search(&source) {
            self.rev_adj[target as usize].insert(index, source);
        }
    }

    pub fn remove_vertex(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = true;
        let forward_list = std::mem::take(&mut self.adj[vertex as usize]);
        for next in forward_list {
            let index = self.rev_adj[next as usize].binary_search(&vertex).unwrap();
            self.rev_adj[next as usize].remove(index);
        }
        let backward_list = std::mem::take(&mut self.rev_adj[vertex as usize]);
        for source in backward_list {
            let index = self.adj[source as usize].binary_search(&vertex).unwrap();
            self.adj[source as usize].remove(index);
        }
    }

    // pub fn initialize_data_structures(&mut self) {
    //     // initialize the heaps
    //     let mut data = Vec::with_capacity(self.total_vertices());
    //     for vertex in 0..self.total_vertices() {
    //         self.current_in_degree[vertex] = self.rev_adj[vertex].len();
    //         let item = MaxItem::new(
    //             vertex,
    //             (self.adj[vertex].len() + self.rev_adj[vertex].len()) as i64,
    //         );
    //         data.push(item);
    //     }
    //     self.max_degree_heap.load(data);

    //     // already look for vertices that are sources or sinks
    //     for vertex in 0..self.total_vertices() {
    //         if self.current_in_degree[vertex] == 0 || self.current_out_degree[vertex] == 0 {
    //             self.sink_source_buffer.push(vertex as u32);
    //         }
    //     }
    // }

    pub fn set_adjacency(&mut self, source: u32, targets: Vec<u32>) {
        for vertex in &targets {
            self.rev_adj[*vertex as usize].push(source);
        }
        self.current_out_degree[source as usize] = targets.len();
        self.adj[source as usize] = targets;
    }

    /// Returns the number of vertices in the original graph
    pub fn total_vertices(&self) -> usize {
        self.adj.len()
    }

    /// Returns the number of remaining vertices
    pub fn vertices(&self) -> usize {
        let mut n = 0;
        for i in 0..self.adj.len() {
            if !self.deleted_vertices[i] {
                n += 1;
            }
        }
        n
    }

    pub fn is_empty(&self) -> bool {
        for i in 0..self.adj.len() {
            if !self.deleted_vertices[i] {
                return false;
            }
        }
        true
    }

    // pub fn num_active_vertices(&self) -> usize {
    //     self.num_active_vertices
    // }

    // /// Requirement: only after disabling a vertex do data structures need to be
    // /// updated. Vertices disabled during kernelizations shall no longer be
    // /// enabled.
    // pub fn disable_vertex(&mut self, vertex: u32) {
    //     // remove from heaps
    //     self.max_degree_heap.decrease_key(MaxItem::new(
    //         vertex as usize,
    //         (self.total_vertices() + 1) as i64,
    //     ));

    //     let val = self.max_degree_heap.extract_min();
    //     debug_assert_eq!(val.unwrap().key() as u32, vertex);

    //     // update synposi
    //     self.active_vertices[vertex as usize] = false;
    //     self.coloring[vertex as usize] = Color::Exhausted;
    //     self.num_active_vertices -= 1;

    //     // update data structures for affected vertices
    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             self.current_out_degree[*incoming as usize] -= 1;

    //             if self.current_out_degree[*incoming as usize] == 0 {
    //                 self.sink_source_buffer.push(*incoming);
    //             }
    //         }
    //     }

    //     for outgoing in &self.adj[vertex as usize] {
    //         if self.active_vertices[*outgoing as usize] {
    //             self.current_in_degree[*outgoing as usize] -= 1;
    //             if self.current_in_degree[*outgoing as usize] == 0 {
    //                 self.sink_source_buffer.push(*outgoing);
    //             }
    //         }
    //     }

    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             let deg = self.current_out_degree[*incoming as usize]
    //                 + self.current_in_degree[*incoming as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
    //         }
    //     }

    //     for outgoing in &self.adj[vertex as usize] {
    //         if self.active_vertices[*outgoing as usize] {
    //             let deg = self.current_out_degree[*outgoing as usize]
    //                 + self.current_in_degree[*outgoing as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*outgoing as usize, deg as i64));
    //         }
    //     }
    // }

    // pub fn enable_vertex(&mut self, vertex: u32) {
    //     self.active_vertices[vertex as usize] = true;
    //     self.coloring[vertex as usize] = Color::Unvisited;
    //     self.num_active_vertices += 1;

    //     for incoming in &self.rev_adj[vertex as usize] {
    //         if self.active_vertices[*incoming as usize] {
    //             self.current_out_degree[*incoming as usize] += 1;
    //             let deg = self.current_out_degree[*incoming as usize];
    //             self.max_degree_heap
    //                 .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
    //         }
    //     }
    // }

    pub fn disable_vertex_post(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = true;
        self.coloring[vertex as usize] = Color::Exhausted;
    }

    pub fn enable_vertex_post(&mut self, vertex: u32) {
        self.deleted_vertices[vertex as usize] = true;
        self.coloring[vertex as usize] = Color::Unvisited;
    }

    // pub fn get_active_vertices(&self) -> Vec<u32> {
    //     let mut result = Vec::new();
    //     for i in 0..self.total_vertices() {
    //         if self.active_vertices[i] {
    //             result.push(i as u32);
    //         }
    //     }
    //     result
    // }

    // pub fn get_disabled_vertices(&self) -> Vec<u32> {
    //     let mut result = Vec::new();
    //     for i in 0..self.total_vertices() {
    //         if !self.active_vertices[i] {
    //             result.push(i as u32)
    //         }
    //     }
    //     result
    // }

    /// Helper algorithm to find cycles in the directed graph.
    fn visit(&self, vertex: usize, coloring: &mut Vec<Color>) -> bool {
        if coloring[vertex] == Color::Exhausted {
            return false;
        }
        if coloring[vertex] == Color::Visited {
            return true;
        }

        coloring[vertex] = Color::Visited;

        for next in &self.adj[vertex] {
            if self.visit(*next as usize, coloring) {
                return true;
            }
        }

        coloring[vertex] = Color::Exhausted;
        return false;
    }

    /// Test whether the graph has a cycle. Simple DFS implementation based on
    /// computing a topological ordering. The graph may consist of several
    /// connected components.
    pub fn is_cyclic(&self) -> bool {
        let mut local_coloring = self.coloring.clone();
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            match local_coloring[i] {
                Color::Unvisited => {
                    if self.visit(i, &mut local_coloring) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Given a set `fvs` of vertices to delete, returns `true` if the
    /// remainder has a cycle somewhere. Simple DFS implementation based on
    /// computing a topological ordering. The graph may consist of several
    /// connected components.
    pub fn has_cycle_with_fvs(&self, fvs: &Vec<u32>) -> bool {
        // keep track of which vertices have been exhaustively visited
        let mut coloring: Vec<_> = vec![Color::Unvisited; self.total_vertices()];
        for vertex in fvs {
            coloring[*vertex as usize] = Color::Exhausted;
        }

        for i in 0..self.total_vertices() {
            if !self.active_vertices[i] {
                continue;
            }
            match coloring[i] {
                Color::Unvisited => {
                    if self.visit(i, &mut coloring) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    // Can be optimized using a heap. Each time a vertex is disabled, we can
    // update the number of in and outgoing edges.
    pub fn max_degree_vertex(&mut self) -> u32 {
        let mut max_deg = 0;
        let mut max_vertex = 0;
        for vertex in 0..self.total_vertices() {
            if !self.deleted_vertices[vertex] {
                let deg_out = self.adj[vertex]
                    .iter()
                    .filter(|u| self.active_vertices[**u as usize])
                    .fold(0, |acc, _| acc + 1) as u32;

                let deg_in = self.rev_adj[vertex]
                    .iter()
                    .filter(|u| self.active_vertices[**u as usize])
                    .fold(0, |acc, _| acc + 1) as u32;

                if deg_in + deg_out > max_deg {
                    max_deg = deg_in + deg_out;
                    max_vertex = vertex;
                }
            }
        }

        max_vertex as u32
    }

    fn has_self_loop(&self) -> bool {
        for i in 0..self.adj.len() {
            if self.adj[i].contains(&(i as u32)) {
                return true;
            }
        }
        return false;
    }

    fn self_loop_reduction(&mut self) -> Vec<u32> {
        let mut forced = Vec::new();
        for i in 0..self.adj.len() {
            if self.adj[i].contains(&(i as u32)) {
                forced.push(i as u32);
                self.remove_vertex(i as u32);
            }
        }
        forced
    }

    fn has_sink_or_source(&self) -> bool {
        // naive implementation to start
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            if list.len() == 0 && !self.deleted_vertices[i] {
                return true;
            }
        }

        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            if list.len() == 0 && !self.deleted_vertices[i] {
                return true;
            }
        }
        return false;
    }

    fn has_single_outgoing(&self) -> bool {
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] {
                if list[0] != i as u32 {
                    return true;
                }
            }
        }
        return false;
    }

    fn has_single_incoming(&self) -> bool {
        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] {
                if list[0] != i as u32 {
                    return true;
                }
            }
        }
        return false;
    }

    fn single_incoming_reduction(&mut self) {
        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] {
                let source = *list.first().unwrap();
                if source == i as u32 {
                    // self-loop
                    continue;
                }

                // mark the vertex as deleted
                self.deleted_vertices[i] = true;

                // get the targets
                let nexts = self.adj[i].clone();

                // already erase the adjacency list
                self.adj[i].clear();

                // vertex i is located in the forward adjacency list of the
                // source
                let index = self.adj[source as usize]
                    .binary_search(&(i as u32))
                    .unwrap();
                self.adj[source as usize].remove(index);

                // redirect edges
                for next in nexts {
                    let v_index = self.rev_adj[next as usize]
                        .binary_search(&(i as u32))
                        .unwrap();
                    self.rev_adj[next as usize].remove(v_index);
                    self.add_arc(source, next);
                }
                self.rev_adj[i].clear();
            }
        }
    }

    fn single_outgoing_reduction(&mut self) {
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] {
                // get the single target
                let target = *list.first().unwrap();
                if target == i as u32 {
                    // self-loop
                    continue;
                }
                // mark the vertex as deleted
                self.deleted_vertices[i] = true;

                // get the sources & clear
                let sources = self.rev_adj[i].clone();
                self.rev_adj[i].clear();
                let index = self.rev_adj[target as usize]
                    .binary_search(&(i as u32))
                    .unwrap();
                self.rev_adj[target as usize].remove(index);

                for source in sources {
                    let v_index = self.adj[source as usize]
                        .binary_search(&(i as u32))
                        .unwrap();
                    self.adj[source as usize].remove(v_index);
                    self.add_arc(source, target);
                }
                self.adj[i].clear();
            }
        }
    }

    fn sink_or_source_reduction(&mut self) {
        for i in 0..self.adj.len() {
            if self.adj[i].len() == 0 && !self.deleted_vertices[i] {
                self.remove_vertex(i as u32);
            }
        }

        for i in 0..self.rev_adj.len() {
            if self.rev_adj[i].len() == 0 && !self.deleted_vertices[i] {
                self.remove_vertex(i as u32);
            }
        }
    }

    fn twin_reduction(&mut self) -> Vec<u32> {
        let mut classes: BTreeMap<Vec<u32>, Vec<u32>> = BTreeMap::new();
        let mut forced = Vec::new();

        let mut has_twins = false;
        for i in 0..self.adj.len() {
            if self.deleted_vertices[i] {
                continue;
            }

            let mut list = self.adj[i].clone();
            // closed neighborhood
            list.push(i as u32);
            list.sort();

            if let Some(class) = classes.get_mut(&list) {
                class.push(i as u32);
                has_twins = true;
            } else {
                classes.insert(list, Vec::new());
            }
        }

        if has_twins || true {
            for (_, twins) in classes {
                // efficiently remove a whole lot of vertices
                for vertex in twins {
                    self.remove_vertex(vertex);
                    forced.push(vertex);
                }
            }
        }
        forced
    }
}

pub trait Reducable {
    fn reduce(&mut self) -> Vec<u32>;
}

impl Reducable for Graph {
    fn reduce(&mut self) -> Vec<u32> {
        let mut reduced = true;
        let mut forced = Vec::new();
        while reduced {
            reduced = false;
            if self.has_sink_or_source() {
                self.sink_or_source_reduction();
                reduced = true;
                continue;
            }
            if self.has_single_outgoing() {
                self.single_outgoing_reduction();
                reduced = true;
                continue;
            }
            if self.has_single_incoming() {
                self.single_incoming_reduction();
                reduced = true;
                continue;
            }
            if self.has_self_loop() {
                forced.append(&mut self.self_loop_reduction());
                reduced = true;
                continue;
            }

            let mut twins = self.twin_reduction();
            if twins.len() != 0 {
                reduced = true;
                forced.append(&mut twins);
            }
        }
        forced
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} 0 0\n", self.total_vertices())?;
        for list in &self.adj {
            let mut first = true;
            for i in 0..list.len() {
                if self.active_vertices[list[i] as usize] {
                    if first {
                        write!(f, "{}", list[i] + 1)?;
                        first = false;
                    } else {
                        write!(f, " {}", list[i] + 1)?;
                    }
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pace_example_graph() -> Graph {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(0, 2);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 0);
        graph
    }


    #[test]
    fn is_cyclic_test_001() {
        let graph = pace_example_graph();
        assert!(graph.is_cyclic());
    }

    #[test]
    fn is_cyclic_test_002() {
        let mut graph = pace_example_graph();
        graph.remove_vertex(1);
        assert!(graph.is_cyclic());
    }

    #[test]
    fn is_cyclic_test_003() {
        let mut graph = pace_example_graph();
        graph.remove_vertex(0);
        assert!(!graph.is_cyclic());
    }

    #[test]
    fn has_fvs_cycle_test_001() {
        let graph = pace_example_graph();
        let fvs = vec![];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_fvs_cycle_test_002() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_fvs_cycle_test_003() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_fvs_cycle_test_004() {
        let graph = pace_example_graph();
        let fvs = vec![0];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }

    #[test]
    fn has_fvs_cycle_test_005() {
        let graph = pace_example_graph();
        let fvs = vec![2];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }

    #[test]
    fn has_fvs_cycle_test_006() {
        let graph = pace_example_graph();
        let fvs = vec![3];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }

    #[test]
    fn self_loop_test() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.reduce();
    }
}
