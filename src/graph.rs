use std::iter::Filter;

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

    pub fn set_adjacency(&mut self, source: u32, targets: Vec<u32>) {
        for vertex in &targets {
            self.rev_adj[*vertex as usize].push(source);
        }
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

    pub fn disable_vertex(&mut self, vertex: u32) {
        self.active_vertices[vertex as usize] = false;
        self.coloring[vertex as usize] = Color::Exhausted;
        self.num_active_vertices -= 1;
    }

    pub fn enable_vertex(&mut self, vertex: u32) {
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
    pub fn max_degree_vertex(&self) -> u32 {
        let mut max_deg = 0;
        let mut max_vertex = 0;
        for vertex in 0..self.total_vertices() {
            if self.active_vertices[vertex] {
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

    // fn sink_source_reduction(&mut self) -> bool {
    //     // definitely use a heap to keep track of vertices that are either
    //     // or sinks
    //     for vertex in self.total_vertices() {
    //         if !self.active_vertices[vertex] {}
    //     }
    // }

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

trait Reducable {
    fn reduce(&mut self);
}

impl Reducable for Graph {
    fn reduce(&mut self) {
        let mut reduced = true;
        while reduced {
            // reduced |= self.sink_source_reduction();
        }
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
