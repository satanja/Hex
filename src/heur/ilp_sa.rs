use crate::util::{Constraint, RangeSet};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng, SeedableRng,
};
use rustc_hash::{FxHashMap, FxHashSet};

struct SimulatedAnnealingILP {
    constraints: Vec<Constraint>,
    adj: Vec<Vec<usize>>,
    states: Vec<bool>,
    rng: StdRng,
    satisfied: RangeSet,
}

enum Delta {
    Infeasible,
    Cost(i32, Option<Vec<u32>>),
}

impl SimulatedAnnealingILP {
    fn new(constraints: &Vec<Constraint>, variables: usize) -> SimulatedAnnealingILP {
        let mut adj = vec![Vec::new(); variables];
        for i in 0..constraints.len() {
            let constraint = &constraints[i];
            for variable in constraint.variables() {
                adj[*variable as usize].push(i);
            }
        }
        let states = vec![true; variables];

        SimulatedAnnealingILP {
            constraints: constraints.clone(),
            adj,
            states,
            rng: StdRng::seed_from_u64(0),
            satisfied: (0..variables as u32).collect(),
        }
    }

    /// Randomly picks a satisfied variable to set to false
    fn random_move(&mut self) -> u32 {
        let i = self.rng.gen_range(0..self.satisfied.len());
        self.satisfied[i]
    }

    /// After flipping `variable` retrieve all constraints that have become
    /// unsatisfied
    fn get_unsatisfied(&self, variable: u32) -> Vec<usize> {
        let mut invalid = Vec::new();
        for i in &self.adj[variable as usize] {
            if !self.constraint_is_satisfied(*i) {
                invalid.push(*i);
            }
        }
        invalid
    }

    /// Determine whether a constraint is satisfied or not
    fn constraint_is_satisfied(&self, constraint_index: usize) -> bool {
        let constraint = &self.constraints[constraint_index];
        let mut sum = 0;
        for variable in constraint.variables() {
            if self.states[*variable as usize] {
                sum += 1;
            }
        }
        sum >= constraint.lower_bound()
    }

    /// Temporarily flips a variable, and computes a set of variables to also
    /// flip to satisfy the ILP again, or `Delta::Infeasible` if there does not
    /// exist such a set.  
    fn delta(&mut self, variable: u32) -> Delta {
        self.flip_variable(variable);
        let unsatisfied = self.get_unsatisfied(variable);

        if unsatisfied.is_empty() {
            self.flip_variable(variable);
            return Delta::Cost(-1, None);
        }

        // Simple greedy heuristic as a first implementation
        let candidate_variables = self.get_candidate_variables(&unsatisfied, variable);
        let mut covered_variables = FxHashSet::default();
        let mut counts: FxHashMap<_, _> = unsatisfied.into_iter().map(|c| (c, 0)).collect();

        for (k, v) in &mut counts {
            for variable in self.constraints[*k].variables() {
                if self.states[*variable as usize] {
                    *v += 1;
                }
            }
        }

        while !counts.is_empty() {
            let mut max_variable = 0;
            let mut max_hit = Vec::new();

            // Determine the variable that hits the most unsatisfied constraints
            for variable in &candidate_variables {
                // Skip variables that have already been included
                if covered_variables.contains(variable) {
                    continue;
                }

                let mut hit = Vec::new();
                for j in &self.adj[*variable as usize] {
                    if counts.contains_key(&j) {
                        hit.push(*j);
                    }
                }

                if hit.len() > max_hit.len() {
                    max_variable = *variable;
                    max_hit = hit;
                }
            }
            // Update the values
            for constraint in &self.adj[max_variable as usize] {
                if let Some(count) = counts.get_mut(constraint) {
                    *count += 1;
                }
            }

            // Include the variable in the solution to fix and remove the set
            // of constraints it hits (if the constraint is then satisfied)
            covered_variables.insert(max_variable);
            for constraint in max_hit {
                let count = counts.get_mut(&constraint).unwrap();
                if *count == self.constraints[constraint].lower_bound() {
                    counts.remove(&constraint);
                } else {
                    *count += 1;
                }
            }
        }

        self.flip_variable(variable);

        let to_fix: Vec<_> = covered_variables.into_iter().collect();
        let delta = self.delta_to_repair(&to_fix, variable);
        Delta::Cost(delta, Some(to_fix))
    }

    /// Determines the cost of flipping a set of variables. Variables set to
    /// true decrease the cost by 1, variables set to false increase the cost
    /// by 1.
    fn delta_to_repair(&self, to_fix: &[u32], moved_variable: u32) -> i32 {
        let mut cost = 0;
        for variable in to_fix {
            if self.states[*variable as usize] {
                cost -= 1;
            } else {
                cost += 1;
            }
        }

        if self.states[moved_variable as usize] {
            cost -= 1;
        } else {
            cost += 1;
        }
        cost
    }

    /// Given a set of unsatisfied constraints, compute the set of variables
    /// that are set to false in the constraints.
    fn get_candidate_variables(&self, unsatisfied: &[usize], variable: u32) -> Vec<u32> {
        let mut set = FxHashSet::default();
        for constraint in unsatisfied {
            for v in self.constraints[*constraint].variables() {
                if variable != *v {
                    set.insert(*v);
                }
            }
        }
        set.into_iter().collect()
    }

    /// Flips a variable from 0 to 1, or from 1 to 0.
    fn flip_variable(&mut self, variable: u32) {
        self.states[variable as usize] ^= true;
    }

    /// Flips all variables in `to_fix`.
    fn flip_variables(&mut self, to_fix: &[u32]) {
        for variable in to_fix {
            self.flip_variable(*variable);
        }
    }

    /// Flips `variable` and all variables in `to_fix`, and fixes the set of
    /// satisfied variables
    fn apply_move(&mut self, variable: u32, to_fix: &[u32]) {
        self.flip_variable(variable);
        self.flip_variables(to_fix);

        let mut new_satisifed = RangeSet::new(self.adj.len());
        for i in 0..self.states.len() {
            if self.states[i] {
                new_satisifed.insert(i as u32);
            }
        }
        self.satisfied = new_satisifed;
    }

    /// Retrieves the current solution.
    fn get_solution(&self) -> Vec<u32> {
        let mut solution = Vec::new();
        for i in 0..self.states.len() {
            if self.states[i] {
                solution.push(i as u32);
            }
        }
        solution
    }
}

pub fn ilp_upper_bound(constraints: &Vec<Constraint>, variables: usize) -> Vec<u32> {
    let mut ilp = SimulatedAnnealingILP::new(constraints, variables);
    let mut best_solution: Vec<_> = (0..variables as u32).collect();
    let iter = variables * 5;
    let ud = Uniform::new(0., 1.);
    let mut temp = 0.99;
    const ALPHA: f64 = 0.999;

    for _ in 0..iter {
        let variable = ilp.random_move();
        if let Delta::Cost(delta, opt_to_fix) = ilp.delta(variable) {
            if delta <= 0 || f64::exp(-delta as f64 / temp) >= ud.sample(&mut ilp.rng) {
                if let Some(to_fix) = opt_to_fix {
                    ilp.apply_move(variable, &to_fix)
                } else {
                    ilp.apply_move(variable, &[]);
                }
                let new_solution = ilp.get_solution();
                if new_solution.len() < best_solution.len() {
                    best_solution = new_solution;
                }
            }
            temp *= ALPHA;
        }
    }
    ilp.get_solution()
}
