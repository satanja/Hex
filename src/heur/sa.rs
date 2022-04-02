use rustc_hash::FxHashMap;

use super::Heuristic;
use crate::{
    graph::{Compressor, Graph, HeuristicReduce},
    util::RangeSet,
};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};

pub struct SimulatedAnnealing {
    graph: Graph,
    mapping: FxHashMap<u32, u32>,
    conf_vtoi: Vec<Option<usize>>,
    conf_itov: Vec<Option<u32>>,
    out_cache: Vec<Option<usize>>,
    in_cache: Vec<Option<usize>>,
    dfvs: RangeSet,
    rng: StdRng,
    reduced: Vec<u32>,
}

impl SimulatedAnnealing {
    fn new(graph: &Graph, reduce_and_compress: bool) -> SimulatedAnnealing {
        let mut clone = graph.clone();
        let mut reduced = Vec::new();
        let (compressed, mapping) = if reduce_and_compress {
            reduced = clone.reduce();
            clone.compress()
        } else {
            let vertices = clone.total_vertices();
            (clone, (0..vertices).map(|v| (v as u32, v as u32)).collect())
        };

        let vertices = compressed.total_vertices();
        SimulatedAnnealing {
            graph: compressed,
            mapping,
            conf_vtoi: vec![None; vertices],
            conf_itov: vec![None; vertices],
            out_cache: vec![None; vertices],
            in_cache: vec![None; vertices],
            dfvs: (0..vertices as u32).collect(),
            rng: StdRng::seed_from_u64(0),
            reduced,
        }
    }

    fn out_index(&mut self, vertex: &u32) -> usize {
        // maybe we should cache this?
        // if let Some(index) = self.out_cache[*vertex as usize] {
        //     return index;
        // }

        let outgoing = self.graph.get_outgoing(vertex);
        let mut min = self.conf_vtoi.len();
        for vertex in outgoing {
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                min = std::cmp::min(min, index);
            }
        }
        // self.out_cache[*vertex as usize] = Some(min);
        min
    }

    fn conflicts_out_size(&self, vertex: &u32, index: usize) -> usize {
        let outgoing = self.graph.get_outgoing(vertex);
        let mut result = 0;
        for vertex in outgoing {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index < index {
                    result += 1;
                }
            }
        }
        result
    }

    fn conflicts_out(&self, vertex: &u32, index: usize) -> Vec<u32> {
        let outgoing = self.graph.get_outgoing(vertex);
        let mut result = Vec::with_capacity(outgoing.len());
        for vertex in outgoing {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index < index {
                    result.push(*vertex);
                }
            }
        }
        result
    }

    fn in_index(&mut self, vertex: &u32) -> usize {
        // if let Some(index) = self.in_cache[*vertex as usize] {
        //     return index;
        // }

        let incoming = self.graph.get_incoming(vertex);
        let mut max = 0;
        for vertex in incoming {
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                max = std::cmp::max(max, index);
            }
        }

        // self.in_cache[*vertex as usize] = Some(max + 1);
        max + 1
    }

    fn conflicts_in_size(&self, vertex: &u32, index: usize) -> usize {
        let incoming = self.graph.get_incoming(vertex);
        let mut result = 0;
        for vertex in incoming {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index >= index {
                    result += 1;
                }
            }
        }
        result
    }

    fn conflicts_in(&self, vertex: &u32, index: usize) -> Vec<u32> {
        let incoming = self.graph.get_incoming(vertex);
        let mut result = Vec::with_capacity(incoming.len());
        for vertex in incoming {
            if let Some(other_index) = self.conf_vtoi[*vertex as usize] {
                if other_index >= index {
                    result.push(*vertex);
                }
            }
        }
        result
    }

    fn delta(&self, vertex: &u32, index: usize) -> i32 {
        self.conflicts_in_size(vertex, index) as i32 + self.conflicts_out_size(vertex, index) as i32
            - 1
    }

    fn random_move(&mut self) -> (u32, usize, bool) {
        let i = self.rng.gen_range(0..self.dfvs.len());
        let vertex = self.dfvs[i];
        let (m, is_in) = if self.rng.gen_bool(0.5) {
            (self.in_index(&vertex), true)
        } else {
            (self.out_index(&vertex), false)
        };

        (vertex, m, is_in)
    }

    fn apply_move(&mut self, vertex: u32, m: usize, is_in: bool) {
        self.dfvs.remove(&vertex);
        let to_remove = if is_in {
            self.conflicts_out(&vertex, m)
        } else {
            self.conflicts_in(&vertex, m)
        };

        for vertex in &to_remove {
            self.dfvs.insert(*vertex);
            if let Some(index) = self.conf_vtoi[*vertex as usize] {
                self.conf_itov[index] = None;
            }
            self.conf_vtoi[*vertex as usize] = None;

            // for neighbor in self.graph.get_incoming(vertex) {
            //     self.in_cache[*neighbor as usize] = None;
            //     self.out_cache[*neighbor as usize] = None;
            // }

            // for neighbor in self.graph.get_outgoing(vertex) {
            //     self.in_cache[*neighbor as usize] = None;
            //     self.out_cache[*neighbor as usize] = None;
            // }
        }

        // Heap allocations go brrrrrrrrrr....
        let mut new_conf_itov = Vec::with_capacity(self.conf_itov.len());
        for j in 0..m {
            if self.conf_itov[j] != None {
                new_conf_itov.push(self.conf_itov[j]);
            }
        }
        new_conf_itov.push(Some(vertex));
        for j in m..self.conf_itov.len() {
            if self.conf_itov[j] != None {
                new_conf_itov.push(self.conf_itov[j]);
            }
        }

        for _ in new_conf_itov.len()..new_conf_itov.capacity() {
            new_conf_itov.push(None);
        }

        for i in 0..new_conf_itov.len() {
            if let Some(vertex) = new_conf_itov[i] {
                self.conf_vtoi[vertex as usize] = Some(i);
            }
        }

        self.conf_itov = new_conf_itov;
    }

    fn recover_complete_solution(&mut self, mut solution: Vec<u32>) -> Vec<u32> {
        for i in 0..solution.len() {
            let vertex = solution[i];
            let original = *self.mapping.get(&vertex).unwrap();
            solution[i] = original;
        }
        solution.append(&mut self.reduced);
        solution
    }

    fn get_solution(&self) -> Vec<u32> {
        self.dfvs.clone_set()
    }

    fn upper_bound(&mut self, graph: &Graph) -> Vec<u32> {
        if !self.graph.is_empty() {
            const TEMPERATURE: f64 = 0.6;
            const ALPHA: f64 = 0.99;
            const FAILS: usize = 50;
            let max_mvt = self.graph.total_vertices() * 5;

            let mut temp = TEMPERATURE;
            let mut nb_fail = 0;
            let mut best_len = self.graph.total_vertices();
            let mut best_solution = Vec::new();

            let ud = Uniform::new(0., 1.);

            loop {
                let mut nb_mvt = 0;
                let mut failure = true;
                loop {
                    let (vertex, m, is_in) = self.random_move();
                    let delta = self.delta(&vertex, m);
                    if delta <= 0 || f64::exp(-delta as f64 / temp) >= ud.sample(&mut self.rng) {
                        self.apply_move(vertex, m, is_in);
                        nb_mvt += 1;

                        if self.dfvs.len() < best_len {
                            best_solution = self.get_solution();
                            best_len = best_solution.len();
                            failure = false;
                        }
                    }
                    if nb_mvt == max_mvt {
                        break;
                    }
                }
                if failure {
                    nb_fail += 1;
                } else {
                    nb_fail = 0;
                }
                temp = temp * ALPHA;

                if nb_fail == FAILS {
                    break;
                }
            }
            best_solution = self.recover_complete_solution(best_solution);
            // With the current parameters, we are usually finding a minimal
            // solution anyways, even though we have no guarantee that it is a
            // minimal solution.
            // best_solution = make_minimal(&mut graph.clone(), best_solution);
            best_solution
        } else {
            self.reduced.clone()
        }
    }
}

impl Heuristic for SimulatedAnnealing {
    fn upper_bound(graph: &Graph) -> Vec<u32> {
        let mut sa = SimulatedAnnealing::new(graph, true);
        let solution = sa.upper_bound(graph);
        solution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_index_test_001() {
        let mut graph = Graph::new(7);
        graph.add_arc(0, 3);
        graph.add_arc(1, 3);
        graph.add_arc(2, 3);
        graph.add_arc(3, 4);
        graph.add_arc(3, 5);
        graph.add_arc(3, 6);
        let mut sa = SimulatedAnnealing::new(&graph, false);
        sa.conf_vtoi[0] = Some(4);
        sa.conf_vtoi[1] = Some(12);
        sa.conf_vtoi[2] = Some(7);
        sa.conf_vtoi[4] = Some(10);
        sa.conf_vtoi[5] = Some(8);
        sa.conf_vtoi[6] = Some(2);

        assert_eq!(sa.out_index(&3), 2);
        assert_eq!(sa.in_index(&3), 13);
    }

    #[test]
    fn conflicts_test_001() {
        let mut graph = Graph::new(2);
        graph.add_arc(0, 1);
        graph.add_arc(1, 0);
        let mut sa = SimulatedAnnealing::new(&graph, false);
        sa.upper_bound(&graph);
    }
}
