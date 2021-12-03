use std::iter::Filter;


pub struct Graph {
    /// The list of vertices which are not deleted.
    active_vertices: Vec<bool>,

    ///
    num_active_vertices: usize,

    coloring: Vec<Color>,

    /// The adjancency list representation of the graph.
    adj: Vec<Vec<u32>>,

    /// The adjacency list representation of the reversed graph.
    rev_adj: Vec<Vec<u32>>,
}

#[derive(Clone, PartialEq, Eq)]
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
        self.active_vertices[vertex as usize] = false;
        self.coloring[vertex as usize] = Color::Unvisited;
        self.num_active_vertices += 1;
    }

    pub fn get_active_vertices(&self) -> Vec<usize> {
        let mut result = Vec::new();
        for i in 0..self.total_vertices() {
            if self.active_vertices[i] {
                result.push(i);
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
    pub fn has_cycle(&self) -> bool {
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