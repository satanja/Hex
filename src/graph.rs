use crate::util::{self, algorithms::intersection};
use core::fmt;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    io::{BufWriter, Write},
    process::ChildStdin,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    // /// The list of vertices which are not deleted.
    active_vertices: Vec<bool>,

    // ///
    // num_active_vertices: usize,
    coloring: Vec<Color>,

    /// The adjacency list representation of the graph.
    adj: Vec<Vec<u32>>,

    /// The adjacency list representation of the reversed graph.
    rev_adj: Vec<Vec<u32>>,

    // /// Out-degree heap
    // max_degree_heap: Heap<MaxItem>,

    // /// Current out-degree of each vertex
    // current_out_degree: Vec<usize>,

    // /// Current in-degree of each vertex
    // current_in_degree: Vec<usize>,

    // /// H
    // sink_source_buffer: Vec<u32>,

    // sinks_or_sources: RangeSet,

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
            // num_active_vertices,
            coloring,
            adj,
            rev_adj,
            // max_degree_heap: Heap::new(0),
            // current_out_degree: vec![0; vertices],
            // current_in_degree: vec![0; vertices],
            // sink_source_buffer: Vec::new(),
            // sinks_or_sources: RangeSet::new(vertices),
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

    pub fn remove_vertices(&mut self, vertices: &Vec<u32>) {
        let mut affected_vertices_forward = FxHashSet::default();
        let mut affected_vertices_back = FxHashSet::default();
        for singleton in vertices {
            for target in &self.adj[*singleton as usize] {
                affected_vertices_forward.insert(*target);
            }
            for source in &self.rev_adj[*singleton as usize] {
                affected_vertices_back.insert(*source);
            }
        }
        for vertex in affected_vertices_forward {
            // u -> vertex, where u in vertices, so look at reverse adjacency list
            let list = std::mem::take(&mut self.rev_adj[vertex as usize]);
            let reduced = util::algorithms::difference(&list, vertices);
            self.rev_adj[vertex as usize] = reduced;
        }

        for vertex in affected_vertices_back {
            // vertex -> u, where u in vertices, so look at the forward adjacency list
            let list = std::mem::take(&mut self.adj[vertex as usize]);
            let reduced = util::algorithms::difference(&list, vertices);
            self.adj[vertex as usize] = reduced;
        }

        for vertex in vertices {
            self.adj[*vertex as usize].clear();
            self.rev_adj[*vertex as usize].clear();
            self.deleted_vertices[*vertex as usize] = true;
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
        // self.current_out_degree[source as usize] = targets.len();
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

    pub fn lower_bound(&self) -> usize {
        let stars = self.stars();
        debug_assert!(stars.len() % 2 == 0);
        stars.len() / 2
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
        self.deleted_vertices[vertex as usize] = false;
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

    /// Given a set `fvs` of vertices to delete, returns `false` if the
    /// remainder has a cycle somewhere, returns true otherwise.
    ///  Simple DFS implementation based on
    /// computing a topological ordering. The graph may consist of several
    /// connected components.
    pub fn is_acyclic_with_fvs(&self, fvs: &Vec<u32>) -> bool {
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
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }

    fn recover_cycle(vertex: u32, dest: u32, pred: &Vec<Option<u32>>) -> Vec<u32> {
        let mut path = Vec::new();
        let mut current_vertex = vertex;
        while current_vertex != dest {
            path.push(current_vertex);
            if pred[current_vertex as usize] == None {
                println!("shit");
            }
            current_vertex = pred[current_vertex as usize].unwrap();
        }
        path.push(dest);
        path.reverse();
        path
    }

    fn dfs_find_cycle(
        &self,
        vertex: usize,
        coloring: &mut Vec<Color>,
        pred: &mut Vec<Option<u32>>,
    ) -> Option<Vec<u32>> {
        if coloring[vertex] == Color::Exhausted {
            return None;
        }

        coloring[vertex] = Color::Visited;

        for next in &self.adj[vertex] {
            if coloring[*next as usize] == Color::Visited {
                return Some(Graph::recover_cycle(vertex as u32, *next, pred));
            }

            pred[*next as usize] = Some(vertex as u32);
            if let Some(cycle) = self.dfs_find_cycle(*next as usize, coloring, pred) {
                return Some(cycle);
            }
        }

        coloring[vertex] = Color::Exhausted;
        return None;
    }

    pub fn find_cycle_with_fvs(&self, fvs: &Vec<u32>) -> Option<Vec<u32>> {
        let mut coloring = vec![Color::Unvisited; self.total_vertices()];
        let mut pred: Vec<Option<u32>> = vec![None; self.total_vertices()];

        for vertex in fvs {
            coloring[*vertex as usize] = Color::Exhausted;
        }

        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            match coloring[i] {
                Color::Unvisited => {
                    if let Some(cycle) = self.dfs_find_cycle(i, &mut coloring, &mut pred) {
                        return Some(cycle);
                    }
                }
                _ => {}
            }
        }
        None
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

    fn strong_connect(
        &self,
        vertex: usize,
        stack: &mut Vec<usize>,
        index_vec: &mut Vec<i32>,
        index: &mut i32,
        low: &mut Vec<i32>,
        comp: &mut Vec<i32>,
        components: &mut i32,
    ) {
        let mut work_stack = vec![(vertex, 0)];
        while let Some((u, j)) = work_stack.pop() {
            if j == 0 {
                index_vec[u] = *index;
                *index += 1;
                low[u] = index_vec[u];
                stack.push(u);
            }
            let mut recurse = false;
            for i in j as usize..self.adj[u].len() {
                let next = self.adj[u][i];
                if index_vec[next as usize] == -1 {
                    work_stack.push((u, i + 1));
                    work_stack.push((next as usize, 0));
                    recurse = true;
                    break;
                } else if comp[next as usize] == -1 {
                    low[u] = std::cmp::min(low[u], index_vec[next as usize]);
                }
            }
            if !recurse {
                if low[u] == index_vec[u] {
                    while let Some(prev) = stack.pop() {
                        comp[prev] = *components;
                        if prev == u {
                            break;
                        }
                    }
                    *components += 1;
                }
                if work_stack.len() != 0 {
                    let (up, _) = work_stack.last().unwrap();
                    low[*up] = std::cmp::min(low[*up], low[u]);
                }
            }
        }
    }

    pub fn tarjan(&self, always_report: bool) -> Option<Vec<Vec<u32>>> {
        let mut index_vec = vec![-1; self.total_vertices()];
        let mut index = 0;
        let mut low = vec![0; self.total_vertices()];
        let mut comp = vec![-1; self.total_vertices()];
        let mut stack = Vec::new();
        let mut components = 0;

        let mut modified = always_report;
        for vertex in 0..self.total_vertices() {
            if index_vec[vertex] == -1 && !self.deleted_vertices[vertex] {
                modified = true;
                self.strong_connect(
                    vertex,
                    &mut stack,
                    &mut index_vec,
                    &mut index,
                    &mut low,
                    &mut comp,
                    &mut components,
                )
            }
        }

        if modified {
            let mut partition = vec![Vec::new(); components as usize];
            for vertex in 0..self.total_vertices() {
                if self.deleted_vertices[vertex] {
                    continue;
                }
                partition[comp[vertex] as usize].push(vertex as u32);
            }
            Some(partition)
        } else {
            None
        }
    }

    fn scc_reduction(&mut self) -> bool {
        let res = self.tarjan(false);
        if res == None {
            return false;
        }

        // only 1 SSC => no point in continuing the reduction
        let components = res.unwrap();
        if components.len() == 1 {
            return false;
        }

        let mut result = false;
        let mut singletons = Vec::new();
        for component in &components {
            if component.len() == 1 {
                let vertex = component[0];
                // may incorrectly pick up self loops
                if !self.adj[vertex as usize].contains(&vertex) {
                    result = true;
                    singletons.push(component[0]);
                }
            }
        }
        singletons.sort();
        self.remove_vertices(&singletons);

        // compute the induced graph by parts of the strongly connected components
        // SCCs may share edges, but they're irrelevant
        // let mut removed_edges = false;
        // for component in components {
        //     if component.len() == 1 {
        //         continue;
        //     }

        //     // TODO possibly optimize
        //     for vertex in &component {
        //         let mut new_adj = Vec::new();
        //         for j in 0..self.adj[*vertex as usize].len() {
        //             let neighbor = self.adj[*vertex as usize][j];
        //             if component.contains(&neighbor) {
        //                 new_adj.push(neighbor);
        //             } else {
        //                 // we found a neighbor not beloning to the component
        //                 // we remove at least one edge from the graph
        //                 removed_edges = true;
        //             }
        //         }
        //         self.adj[*vertex as usize] = new_adj;
        //     }
        // }

        // // only change the reverse adjacency list if actual progress has been
        // // made
        // if removed_edges {
        //     // rebuild reverse adj
        //     self.rev_adj = vec![Vec::new(); self.total_vertices()];
        //     for i in 0..self.adj.len() {
        //         for j in 0..self.adj[i].len() {
        //             let target = self.adj[i][j];
        //             if let Err(index) = self.rev_adj[target as usize].binary_search(&(i as u32)) {
        //                 self.rev_adj[target as usize].insert(index, i as u32);
        //             }
        //         }
        //     }
        // }
        result
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

    fn has_single_outgoing(&self) -> bool {
        for i in 0..self.adj.len() {
            let list = &self.adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] && list[0] != i as u32 {
                return true;
            }
        }
        return false;
    }

    fn has_single_incoming(&self) -> bool {
        for i in 0..self.rev_adj.len() {
            let list = &self.rev_adj[i];
            if list.len() == 1 && !self.deleted_vertices[i] && list[0] != i as u32 {
                return true;
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

    fn has_empty_vertex(&self) -> bool {
        for i in 0..self.adj.len() {
            if self.adj[i].len() == 0 && self.rev_adj[i].len() == 0 && !self.deleted_vertices[i] {
                return true;
            }
        }
        false
    }

    fn empty_vertices(&mut self) {
        for i in 0..self.adj.len() {
            if self.adj[i].len() == 0 && self.rev_adj[i].len() == 0 && !self.deleted_vertices[i] {
                self.deleted_vertices[i] = true;
            }
        }
    }

    /// Finds vertices contained in a 2-cycle and all its neighbors that are
    /// included in the 2-cycles
    pub fn stars(&self) -> Vec<(u32, Vec<u32>)> {
        let mut count = vec![Vec::new(); self.total_vertices()];
        for i in 0..self.total_vertices() {
            if self.deleted_vertices[i] {
                continue;
            }
            for j in 0..self.adj[i].len() {
                let t = self.adj[i][j];
                debug_assert!(!self.deleted_vertices[t as usize]);
                if self.adj[t as usize].contains(&(i as u32)) {
                    count[i].push(t);
                }
            }
        }
        let mut stars = Vec::new();
        for i in 0..count.len() {
            if count[i].len() != 0 {
                let neighborhood = std::mem::take(&mut count[i]);
                stars.push((i as u32, neighborhood));
            }
        }
        stars
    }

    pub fn max_degree_star(&self) -> Option<(u32, Vec<u32>)> {
        let mut stars = self.stars();
        if stars.len() == 0 {
            return None;
        }

        let mut max = stars.pop().unwrap();
        for star in stars {
            if star.1.len() >= max.1.len() {
                max = star;
            }
        }

        Some(max)
    }

    fn star_reduction(&mut self, parameter: usize) -> Option<u32> {
        let stars = self.stars();
        if stars.len() == 0 {
            return None;
        }
        let (v, neighbours) = stars.last().unwrap();
        if neighbours.len() > parameter {
            self.remove_vertex(*v);
            return Some(*v);
        }
        return None;
    }

    fn twin_reduction(&mut self) -> Vec<u32> {
        let mut classes: FxHashMap<Vec<u32>, Vec<u32>> = FxHashMap::default();
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

        if has_twins {
            for (_, mut twins) in classes {
                self.remove_vertices(&twins);
                forced.append(&mut twins);
            }
        }
        forced
    }

    pub fn induced_subgraph(&self, mut subset: Vec<u32>) -> Graph {
        subset.sort();
        let mut induced = self.clone();

        for i in 0..induced.total_vertices() {
            if induced.adj[i].len() == 0 {
                continue;
            }

            let intersect_adj = intersection(&induced.adj[i], &subset);
            let intersect_rev_adj = intersection(&induced.rev_adj[i], &subset);

            induced.adj[i] = intersect_adj;
            induced.rev_adj[i] = intersect_rev_adj;
        }

        induced
    }
}

pub trait Reducable {
    fn reduce(&mut self, upper_bound: usize) -> Option<Vec<u32>>;
}

impl Reducable for Graph {
    fn reduce(&mut self, mut upper_bound: usize) -> Option<Vec<u32>> {
        let mut reduced = true;
        let mut forced = Vec::new();
        while reduced {
            reduced = false;
            if self.scc_reduction() {
                reduced = true;
            }

            if self.has_empty_vertex() {
                self.empty_vertices();
                reduced = true;
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

            upper_bound = std::cmp::min(upper_bound, self.vertices());

            if self.has_self_loop() {
                let mut self_loops = self.self_loop_reduction();
                if self_loops.len() > upper_bound {
                    return None;
                }
                reduced = true;
                upper_bound -= self_loops.len();
                forced.append(&mut self_loops);
                continue;
            }

            let mut twins = self.twin_reduction();
            if twins.len() > upper_bound {
                return None;
            }

            if twins.len() != 0 {
                reduced = true;
                upper_bound -= twins.len();
                forced.append(&mut twins);
            }

            if let Some(vertex) = self.star_reduction(upper_bound) {
                if upper_bound == 0 {
                    return None;
                }
                reduced = true;
                upper_bound -= 1;
                forced.push(vertex);
            }
        }
        Some(forced)
    }
}

pub trait HeuristicReduce {
    fn reduce(&mut self) -> Vec<u32>;
}

impl HeuristicReduce for Graph {
    fn reduce(&mut self) -> Vec<u32> {
        let mut reduced = true;
        let mut forced = Vec::new();
        while reduced {
            reduced = false;
            if self.scc_reduction() {
                reduced = true;
            }

            if self.has_empty_vertex() {
                self.empty_vertices();
                reduced = true;
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
                let mut self_loops = self.self_loop_reduction();
                reduced = true;
                forced.append(&mut self_loops);
                continue;
            }
        }
        forced
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} 0 0", self.total_vertices())?;
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
            writeln!(f)?;
        }
        Ok(())
    }
}
pub trait Undirected {
    fn is_undirected(&self) -> bool;
    fn write_to_stdin(&self, stdin: ChildStdin);
}

impl Undirected for Graph {
    fn is_undirected(&self) -> bool {
        for i in 0..self.adj.len() {
            for j in 0..self.adj[i].len() {
                let u = self.adj[i][j];
                if !self.rev_adj[i].contains(&u) {
                    return false;
                }
            }
        }
        true
    }

    fn write_to_stdin(&self, stdin: ChildStdin) {
        let mut writer = BufWriter::new(stdin);
        let directed_edges = self.adj.iter().fold(0, |x, list| x + list.len());
        let edges = directed_edges / 2;

        writeln!(writer, "p td {} {}", self.total_vertices(), edges).unwrap();
        let edges = self.undir_edge_iter();
        for (u, v) in edges {
            writeln!(writer, "{} {}", u + 1, v + 1).unwrap();
        }
        writer.flush().unwrap();
    }
}

pub trait EdgeIter {
    fn undir_edge_iter(&self) -> UndirEdgeIter;
}

impl EdgeIter for Graph {
    fn undir_edge_iter(&self) -> UndirEdgeIter {
        UndirEdgeIter {
            current_vertex: 0,
            current_neighbor: 0,
            graph: &self,
        }
    }
}

pub struct UndirEdgeIter<'a> {
    current_vertex: usize,
    current_neighbor: usize,
    graph: &'a Graph,
}

impl<'a> Iterator for UndirEdgeIter<'a> {
    type Item = (u32, u32);

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.current_vertex..self.graph.total_vertices() {
            for j in self.current_neighbor..self.graph.adj[i].len() {
                let u = self.graph.adj[i][j];
                if i < u as usize {
                    self.current_vertex = i;
                    self.current_neighbor = j + 1;
                    return Some((i as u32, u));
                }
            }
            self.current_neighbor = 0;
        }
        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeIter;
    use super::Graph;
    use super::Reducable;

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
        assert_eq!(graph.is_acyclic_with_fvs(&fvs), false);
    }

    #[test]
    fn has_fvs_cycle_test_002() {
        let graph = pace_example_graph();
        let fvs = vec![1];
        assert_eq!(graph.is_acyclic_with_fvs(&fvs), false);
    }

    #[test]
    fn has_fvs_cycle_test_004() {
        let graph = pace_example_graph();
        let fvs = vec![0];
        assert_eq!(graph.is_acyclic_with_fvs(&fvs), true);
    }

    #[test]
    fn has_fvs_cycle_test_005() {
        let graph = pace_example_graph();
        let fvs = vec![2];
        assert_eq!(graph.is_acyclic_with_fvs(&fvs), true);
    }

    #[test]
    fn has_fvs_cycle_test_006() {
        let graph = pace_example_graph();
        let fvs = vec![3];
        assert_eq!(graph.is_acyclic_with_fvs(&fvs), true);
    }

    #[test]
    fn self_loop_test() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.reduce(2);
    }

    #[test]
    fn scc_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        let components = graph.tarjan(false).unwrap();
        assert_eq!(components.len(), 2);
    }

    #[test]
    fn scc_test_002() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        graph.add_arc(2, 0);
        let components = graph.tarjan(false).unwrap();
        assert_eq!(components.len(), 1);
    }

    #[test]
    fn ssc_reduction_test_001() {
        let mut graph = Graph::new(4);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 2);
        graph.scc_reduction();

        let mut expected = Graph::new(4);
        expected.add_arc(0, 1);
        expected.add_arc(1, 0);
        expected.add_arc(2, 3);
        expected.add_arc(3, 2);

        // assert_eq!(graph, expected);
    }

    #[test]
    fn ssc_reduction_test002() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.single_incoming_reduction();
        graph.scc_reduction();
        assert_eq!(graph.vertices(), 1);
    }

    #[test]
    fn undir_edge_iter_test_001() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        graph.add_arc(0, 2);
        graph.add_arc(2, 0);
        graph.add_arc(1, 2);
        graph.add_arc(2, 1);
        let iter = graph.undir_edge_iter();
        assert_eq!(iter.count(), 3);
    }

    #[test]
    fn find_cycle_test_001() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        assert_eq!(graph.find_cycle_with_fvs(&vec![]), None);
    }

    #[test]
    fn find_cycle_test_002() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        assert_eq!(graph.find_cycle_with_fvs(&vec![]), Some(vec![0, 1]));
    }

    #[test]
    fn find_cycle_test_003() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 2);

        assert_eq!(graph.find_cycle_with_fvs(&vec![]), Some(vec![2, 3, 4]));
    }

    #[test]
    fn find_cycle_test_004() {
        let mut graph = Graph::new(5);
        graph.add_arc(0, 1);
        graph.add_arc(1, 2);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(4, 2);

        assert_eq!(graph.find_cycle_with_fvs(&vec![2]), None);
    }

    #[test]
    fn find_cycle_test_005() {
        let mut graph = Graph::new(3);
        graph.add_arc(0, 1);
        graph.add_arc(0, 2);
        graph.add_arc(1, 2);
        assert_eq!(graph.find_cycle_with_fvs(&vec![]), None);
    }
}
