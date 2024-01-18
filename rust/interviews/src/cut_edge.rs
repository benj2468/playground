use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
};

#[derive(Debug, Default)]
pub struct Graph<T, E>(HashMap<T, HashSet<(T, E)>>);

impl<T, E> Graph<T, E>
where
    T: PartialEq + Eq + Hash + Clone,
    E: PartialEq + Eq + Hash + Clone,
{
    fn insert_edge(&mut self, edge: (T, T, E)) {
        let (src, dst, name) = edge;
        self.0
            .entry(src.clone())
            .or_default()
            .insert((dst.clone(), name.clone()));
        self.0.entry(dst).or_default().insert((src, name));
    }
}

pub trait CutEdge<T> {
    /// Assumptions:
    /// 1. The input graph is fully connected
    /// 2. Graph is well formed
    fn cut_edges(&self) -> HashSet<(&T, &T)>;
}

impl<T, E> CutEdge<T> for Graph<T, E>
where
    T: PartialEq + Eq + Hash + Debug,
{
    fn cut_edges(&self) -> HashSet<(&T, &T)> {
        if self.0.is_empty() {
            return Default::default();
        }
        // Backtracking will help us understand if we have found a cycle.
        // This is a hashmap, so will only ever hold as many as nodes in graph: O(n)
        let mut backtracking: HashMap<&T, &T> = HashMap::default();

        // Resulting set will start as all of the edges in the entire graph. O(m)
        let mut res: HashSet<_> = self
            .0
            .iter()
            .flat_map(|(s, neighbors)| neighbors.iter().map(move |(n, _)| (s, n)))
            .collect();

        let mut remove_edge = |a, b| {
            res.remove(&(a, b));
            res.remove(&(b, a));
        };

        // Visited can hold the entire graph: O(n)
        let mut visited = HashSet::<&T>::default();

        // By def, we will only visit a node once, so this is also O(n)
        let mut to_visit = VecDeque::new();
        to_visit.push_back(self.0.iter().next().unwrap().0);

        // Pop from the from (nature of DFS)
        while let Some(cur) = to_visit.pop_front() {
            // Loop over neighbors
            for (neighbor, _) in self.0.get(cur).unwrap() {
                // [IF] The node that got us to `cur` is the same as `neighbor`, then we are traversing
                // the same path again, which we DO NOT want to do (this makes undirected graphs actually not cycles)
                if backtracking
                    .get(cur)
                    .map(|back| back == &neighbor)
                    .unwrap_or_default()
                {
                    continue;
                }

                // [IF] we have ALREADY seen this node, then we must have a cycle, 
                // since we traversed two paths to get to it
                if visited.contains(neighbor) {
                    // Traverse up the backtracking tree 
                    // removing edges as we go through the pat that got us back to this nodes
                    let mut step = cur;
                    remove_edge(neighbor, step);
                    while let Some(back) = backtracking.get(step) {
                        remove_edge(back, step);
                        step = back;
                    }

                    continue;
                }
                // Push to the front (nature of DFS)
                to_visit.push_front(neighbor);
                // Inform backtracking of how we got here
                backtracking.insert(neighbor, cur);
            }

            // Complete our visit
            visited.insert(cur);
        }

        res
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn simple() {
        let mut graph = Graph::default();
        graph.insert_edge((1, 2, "n1"));
        graph.insert_edge((2, 3, "n3"));
        graph.insert_edge((1, 3, "n4"));
        graph.insert_edge((1, 4, "n1"));
        graph.insert_edge((5, 4, "n1"));
        graph.insert_edge((5, 2, "n1"));

        println!("{:?}", graph.cut_edges())
    }
}
