use std::{iter::Filter, usize};

use crate::util::{Heap, KeyValue, MaxItem, MinItem, RangeSet};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    /// The list of vertices which are not deleted.
    active_vertices: Vec<bool>,

    ///
    num_active_vertices: usize,

    coloring: Vec<Color>,

    /// The adjacency list representation of the graph.
    adj: Vec<Vec<u32>>,

    /// The adjacency list representation of the reversed graph.
    rev_adj: Vec<Vec<u32>>,

    /// Out-degree heap
    max_degree_heap: Heap<MaxItem>,

    /// Current out-degree of each vertex
    current_out_degree: Vec<usize>,

    /// Current in-degree of each vertex
    current_in_degree: Vec<usize>,

    /// H
    sink_source_buffer: Vec<u32>,

    sinks_or_sources: RangeSet,
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

    pub fn initialize_data_structures(&mut self) {
        // initialize the heaps
        let mut data = Vec::with_capacity(self.total_vertices());
        for vertex in 0..self.total_vertices() {
            self.current_in_degree[vertex] = self.rev_adj[vertex].len();
            let item = MaxItem::new(
                vertex,
                (self.adj[vertex].len() + self.rev_adj[vertex].len()) as i64,
            );
            data.push(item);
        }
        self.max_degree_heap.load(data);

        // already look for vertices that are sources or sinks
        for vertex in 0..self.total_vertices() {
            if self.current_in_degree[vertex] == 0 || self.current_out_degree[vertex] == 0 {
                self.sink_source_buffer.push(vertex as u32);
            }
        }
    }

    pub fn set_adjacency(&mut self, source: u32, targets: Vec<u32>) {
        for vertex in &targets {
            self.rev_adj[*vertex as usize].push(source);
        }
        self.current_out_degree[source as usize] = targets.len();
        self.adj[source as usize] = targets;
    }

    /// Returns the number (remaining) of vertices in the graph
    pub fn total_vertices(&self) -> usize {
        // TODO compute the actual number of remaining vertices in the graph
        self.adj.len()
    }

    pub fn num_active_vertices(&self) -> usize {
        self.num_active_vertices
    }

    /// Requirement: only after disabling a vertex do data structures need to be
    /// updated. Vertices disabled during kernelizations shall no longer be
    /// enabled.
    pub fn disable_vertex(&mut self, vertex: u32) {
        // remove from heaps
        self.max_degree_heap.decrease_key(MaxItem::new(
            vertex as usize,
            (self.total_vertices() + 1) as i64,
        ));
        
        let val = self.max_degree_heap.extract_min();
        debug_assert_eq!(val.unwrap().key() as u32, vertex);
        
        // update synposi
        self.active_vertices[vertex as usize] = false;
        self.coloring[vertex as usize] = Color::Exhausted;
        self.num_active_vertices -= 1;

        // update data structures for affected vertices
        for incoming in &self.rev_adj[vertex as usize] {
            if self.active_vertices[*incoming as usize] {
                self.current_out_degree[*incoming as usize] -= 1;

                if self.current_out_degree[*incoming as usize] == 0 {
                    self.sink_source_buffer.push(*incoming);
                }
            }
        }

        for outgoing in &self.adj[vertex as usize] {
            if self.active_vertices[*outgoing as usize] {
                self.current_in_degree[*outgoing as usize] -= 1;
                if self.current_in_degree[*outgoing as usize] == 0 {
                    self.sink_source_buffer.push(*outgoing);
                }
            }
        }

        for incoming in &self.rev_adj[vertex as usize] {
            if self.active_vertices[*incoming as usize] {
                let deg = self.current_out_degree[*incoming as usize]
                    + self.current_in_degree[*incoming as usize];
                self.max_degree_heap
                    .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
            }
        }

        for outgoing in &self.adj[vertex as usize] {
            if self.active_vertices[*outgoing as usize] {
                let deg = self.current_out_degree[*outgoing as usize]
                    + self.current_in_degree[*outgoing as usize];
                self.max_degree_heap
                    .decrease_key(MaxItem::new(*outgoing as usize, deg as i64));
            }
        }
    }

    pub fn enable_vertex(&mut self, vertex: u32) {
        self.active_vertices[vertex as usize] = true;
        self.coloring[vertex as usize] = Color::Unvisited;
        self.num_active_vertices += 1;

        for incoming in &self.rev_adj[vertex as usize] {
            if self.active_vertices[*incoming as usize] {
                self.current_out_degree[*incoming as usize] += 1;
                let deg = self.current_out_degree[*incoming as usize];
                self.max_degree_heap
                    .decrease_key(MaxItem::new(*incoming as usize, deg as i64));
            }
        }
    }

    pub fn disable_vertex_post(&mut self, vertex: u32) {
        self.active_vertices[vertex as usize] = false;
        self.coloring[vertex as usize] = Color::Exhausted;
        self.num_active_vertices -= 1;
    }

    pub fn enable_vertex_post(&mut self, vertex: u32) {
        self.active_vertices[vertex as usize] = true;
        self.coloring[vertex as usize] = Color::Unvisited;
        self.num_active_vertices += 1;
    }

    pub fn get_active_vertices(&self) -> Vec<u32> {
        let mut result = Vec::new();
        for i in 0..self.total_vertices() {
            if self.active_vertices[i] {
                result.push(i as u32);
            }
        }
        result
    }

    pub fn get_disabled_vertices(&self) -> Vec<u32> {
        let mut result = Vec::new();
        for i in 0..self.total_vertices() {
            if !self.active_vertices[i] {
                result.push(i as u32)
            }
        }
        result
    }

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
            if !self.active_vertices[i] {
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
        let max = self.max_degree_heap.peek_min().unwrap();
        max.key() as u32
    }

    fn has_sink_or_source(&self) -> bool {
        self.sink_source_buffer.len() != 0
    }

    fn sink_or_source_reduction(&mut self) {
        while let Some(vertex) = self.sink_source_buffer.pop() {
            // See if it is possible not to add already disabled vertices
            if self.active_vertices[vertex as usize] {
                self.sinks_or_sources.insert(vertex);
            }
        }

        while let Some(vertex) = self.sinks_or_sources.pop() {
            self.disable_vertex(vertex as u32);
        }
    }

    // fn contract(&mut self, vertex: u32) {
    //     let target = self.adj[vertex as usize][0];
    //     for source in self.rev_adj[vertex as usize] {
    //         if self.adj.len() <= 25 {
    //             for i in 0..self.adj[source].len() {

    //             }
    //         } else {

    //         }
    //     }
    // }
}

pub trait Reducable {
    fn reduce(&mut self);
}

impl Reducable for Graph {
    fn reduce(&mut self) {
        let mut reduced = true;
        while reduced {
            reduced = false;
            if self.has_sink_or_source() {
                self.sink_or_source_reduction();
                reduced = true;
            }
        }
        let mut k = 0;
        for vertex in 0..self.total_vertices() {
            if self.active_vertices[vertex] {
                if self.current_in_degree[vertex] == 1 && self.current_out_degree[vertex] == 1 {
                    k += 1;
                }
            }
        }
        println!("{}", k);
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
    fn has_cycle_test_001() {
        let graph = pace_example_graph();
        let fvs = vec![];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_cycle_test_002() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_cycle_test_003() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), true);
    }

    #[test]
    fn has_cycle_test_004() {
        let graph = pace_example_graph();
        let fvs = vec![0];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }

    #[test]
    fn has_cycle_test_005() {
        let graph = pace_example_graph();
        let fvs = vec![2];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }

    #[test]
    fn has_cycle_test_006() {
        let graph = pace_example_graph();
        let fvs = vec![3];
        assert_eq!(graph.has_cycle_with_fvs(&fvs), false);
    }
}
