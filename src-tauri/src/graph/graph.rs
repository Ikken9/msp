use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use log::info;
use crate::graph::edge::Edge;
use crate::graph::node::{Node, NodeId};

pub struct Graph {
    pub nodes: HashMap<NodeId, Node>,
    edges: HashMap<(NodeId, NodeId), Edge>
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: &str) -> Result<(), String> {
        if self.nodes.contains_key(id) {
            return Err("Node already exists".to_string());
        }

        self.nodes.insert(id.to_string(), Node::new(id.parse().unwrap()));
        Ok(())
    }

    pub fn add_edge(&mut self, source: NodeId, target: NodeId, cost: u32) -> Result<(), String> {
        if self.edges.contains_key(&(source.clone(), target.clone())) {
            return Err("Edge already exists".to_string());
        }
        self.nodes.insert(source.clone(), Node::new(source.clone()));
        self.nodes.insert(target.clone(), Node::new(target.clone()));

        let edge = Edge::new(source.clone(), target.clone(), cost);
        let reverse_edge = Edge::new(target.clone(), source.clone(), cost);

        self.edges.insert((source.clone(), target.clone()), edge);
        self.edges.insert((target, source), reverse_edge);

        Ok(())
    }

    pub fn remove_node(&mut self, id: &str) -> Result<(), String> {
        if self.nodes.contains_key(id) {
            self.nodes.remove(id);
            return Ok(())
        }

        Err("Node does not exist".to_string())
    }

    pub fn remove_edge(&mut self, source: NodeId, target: NodeId) -> Result<(), String> {
        if self.edges.contains_key(&(source.clone(), target.clone())) {
            self.edges.remove(&(source.clone(), target));
            return Ok(())
        }

        Err("Edge does not exist".to_string())
    }

    pub fn set_node_availability(&mut self, id: &str, status: bool) -> Result<(), String> {
        if let Some(node) = self.nodes.get_mut(id) {
            node.available = status;
            return Ok(())
        }

        Err("Node does not exist".to_string())
    }

    pub fn floyd_warshall(&self) -> (Vec<Vec<usize>>, Vec<Vec<usize>>) {
        let n = self.nodes.len();

        let mut distances = self.build_initial_cost_matrix();
        let mut predecessors = vec![vec![usize::MAX; n]; n];

        let index_map = self.build_index_map();

        for (source, source_index) in &index_map {
            for (target, target_index) in &index_map {
                if source != target && distances[*source_index][*target_index] < usize::MAX {
                    predecessors[*source_index][*target_index] = *source_index;
                }
            }
        }

        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if distances[i][k] < usize::MAX && distances[k][j] < usize::MAX {
                        let new_distance = distances[i][k] + distances[k][j];
                        if new_distance < distances[i][j] {
                            distances[i][j] = new_distance;
                            predecessors[i][j] = predecessors[k][j];
                        }
                    }
                }
            }
        }

        (distances, predecessors)
    }

    pub fn floyd_warshall_map(&self) -> HashMap<(NodeId, NodeId), (Vec<NodeId>, u32)> {
        let nodes: Vec<NodeId> = self.nodes.keys().cloned().collect();
        let mut dist: HashMap<(NodeId, NodeId), u32> = HashMap::new();
        let mut next: HashMap<(NodeId, NodeId), NodeId> = HashMap::new();

        for i in &nodes {
            for j in &nodes {
                if i == j {
                    dist.insert((i.clone(), j.clone()), 0);
                } else if let Some(edge) = self.edges.get(&(i.clone(), j.clone())) {
                    dist.insert((i.clone(), j.clone()), edge.cost);
                    next.insert((i.clone(), j.clone()), j.clone());
                } else {
                    dist.insert((i.clone(), j.clone()), u32::MAX / 2);
                }
            }
        }

        for k in &nodes {
            for i in &nodes {
                for j in &nodes {
                    let ij = dist[&(i.clone(), j.clone())];
                    let ik = dist[&(i.clone(), k.clone())];
                    let kj = dist[&(k.clone(), j.clone())];

                    if ik + kj < ij {
                        dist.insert((i.clone(), j.clone()), ik + kj);
                        next.insert((i.clone(), j.clone()), next[&(i.clone(), k.clone())].clone());
                    }
                }
            }
        }

        let mut paths: HashMap<(NodeId, NodeId), (Vec<NodeId>, u32)> = HashMap::new();
        for i in &nodes {
            for j in &nodes {
                if i != j {
                    if let Some(path) = self.build_path(i, j, &next) {
                        let cost = dist[&(i.clone(), j.clone())];
                        paths.insert((i.clone(), j.clone()), (path, cost));
                    }
                }
            }
        }

        paths
    }

    pub fn dijkstra(&mut self, start: Node) -> HashMap<NodeId, u32> {
        let mut distances: HashMap<NodeId, u32> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut priority_queue = BinaryHeap::new();

        distances.insert(start.id.clone(), 0);

        priority_queue.push(
            State {
                node: start.id.clone(),
                cost: 0
            }
        );

        while let Some(State { node: current_node, cost: current_distance }) = priority_queue.pop() {
            if !visited.insert(current_node.clone()) {
                continue;
            }

            if let Some(v) = self.nodes.get(&current_node) {
                let edges = self.edges
                    .iter()
                    .filter(|&((source, _), _)|source.eq(&v.id.to_string()))
                    .map(|((_, target), edge)| edge);

                for edge in edges {
                    if let Some(next) = self.nodes.get(&edge.target) {
                        let distance = current_distance + edge.cost;

                        if distance < *distances.get(&edge.target).unwrap_or(&u32::MAX) {
                            distances.insert(edge.target.clone(), distance);

                            priority_queue.push(
                                State {
                                    node: next.id.clone(),
                                    cost: distance
                                }
                            );
                        }
                    }
                }
            }
        }

        distances
    }

    pub fn dijkstra_predecessors(&mut self, start: NodeId, target: NodeId) -> Option<(Vec<NodeId>, u32)> {
        let mut distances: HashMap<NodeId, u32> = HashMap::new();
        let mut predecessors: HashMap<NodeId, Option<NodeId>> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

        if let Some(node) = self.nodes.get(&start) {
            distances.insert(node.id.clone(), 0);
            predecessors.insert(node.id.clone(), None);

            if !node.available {
                return None;
            }

            priority_queue.push(State {
                node: node.id.clone(),
                cost: 0,
            });

            while let Some(State { node: current_node, cost: current_distance }) = priority_queue.pop() {
                if let Some(node) = self.nodes.get(&current_node) {
                    if !node.available {
                        continue;
                    }
                } else {
                    continue;
                }

                if let Some(&known_distance) = distances.get(&current_node) {
                    if current_distance > known_distance {
                        continue;
                    }
                }

                if current_node == target {
                    let mut path = Vec::new();
                    let mut current = Some(target.clone());
                    while let Some(node) = current {
                        path.push(node.clone());
                        current = predecessors.get(&node).cloned().unwrap_or(None);
                    }
                    path.reverse();
                    return Some((path, current_distance));
                }

                let edges = self.edges.values()
                    .filter(|edge| edge.source == current_node);

                for edge in edges {
                    if let Some(next_node) = self.nodes.get_mut(&edge.target.clone()) {
                        if !next_node.available {
                            continue;
                        }

                        let distance = current_distance + edge.cost;

                        if distances.get(&next_node.id).map_or(true, |&d| distance < d) {
                            distances.insert(next_node.id.clone(), distance);
                            predecessors.insert(next_node.id.clone(), Some(current_node.clone()));
                            priority_queue.push(State {
                                node: next_node.id.clone(),
                                cost: distance,
                            });
                        }
                    } else {
                        continue;
                    };
                }
            }

            return None
        }

        None
    }

    pub fn dijkstra_re_path(&self, start: &NodeId, target: &NodeId, exclude_nodes: &HashSet<NodeId>) -> Option<Vec<NodeId>> {
        let mut distances: HashMap<NodeId, u32> = HashMap::new();
        let mut predecessors: HashMap<NodeId, NodeId> = HashMap::new();
        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut priority_queue = BinaryHeap::new();

        distances.insert(start.clone(), 0);
        priority_queue.push(State {
            node: start.clone(),
            cost: 0,
        });

        while let Some(State { node: current_node, cost: current_distance }) = priority_queue.pop() {
            if !visited.insert(current_node.clone()) {
                continue;
            }

            if exclude_nodes.contains(&current_node) || !self.is_node_available(&current_node) {
                continue;
            }

            if &current_node == target {
                let mut path = Vec::new();
                let mut current = current_node.clone();
                while let Some(prev) = predecessors.get(&current) {
                    path.push(current.clone());
                    current = prev.clone();
                }
                path.push(start.clone());
                path.reverse();
                return Some(path);
            }

            if let Some(neighbors) = self.get_neighbors(&current_node) {
                for neighbor_id in neighbors.clone() {
                    if exclude_nodes.contains(&neighbor_id) || !self.is_node_available(&neighbor_id) {
                        continue;
                    }

                    let edge = self.get_edge(&current_node, &neighbor_id).unwrap();
                    let distance = current_distance + edge.cost;

                    if distance < *distances.get(&neighbor_id).unwrap_or(&u32::MAX) {
                        distances.insert(neighbor_id.clone(), distance);
                        predecessors.insert(neighbor_id.clone(), current_node.clone());
                        priority_queue.push(State {
                            node: neighbor_id.clone(),
                            cost: distance,
                        });
                    }
                }
            }
        }

        None
    }

    fn is_node_available(&self, node_id: &NodeId) -> bool {
        self.nodes.get(node_id).map_or(false, |node| node.available)
    }

    fn get_neighbors(&self, node_id: &NodeId) -> Option<Vec<NodeId>> {
        if !self.nodes.contains_key(node_id) {
            return None;
        }

        let neighbors = self.edges
            .iter()
            .filter_map(|((source, target), edge)| {
                if source == node_id {
                    Some(target.clone())
                } else if target == node_id {
                    Some(source.clone())
                } else {
                    None
                }
            })
            .collect();

        Some(neighbors)
    }

    fn get_edge(&self, source: &NodeId, target: &NodeId) -> Option<&Edge> {
        self.edges.get(&(source.clone(), target.clone()))
            .or_else(|| self.edges.get(&(target.clone(), source.clone())))
    }

    pub fn build_initial_cost_matrix(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let index_map = self.build_index_map();
        let mut matrix = vec![vec![usize::MAX; n]; n];

        for i in 0..n {
            matrix[i][i] = 0;
        }

        for edge in self.edges.values() {
            let source_index = *index_map.get(&edge.source).unwrap();
            let target_index = *index_map.get(&edge.target).unwrap();
            matrix[source_index][target_index] = edge.cost as usize;
        }

        matrix
    }

    fn build_index_map(&self) -> HashMap<String, usize> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id.clone(), i)).collect()
    }

    pub fn get_node_ids(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.nodes.keys().into_iter().cloned().collect();
        ids.sort();
        ids
    }

    fn build_path(&self, i: &NodeId, j: &NodeId, next: &HashMap<(NodeId, NodeId), NodeId>) -> Option<Vec<NodeId>> {
        if !next.contains_key(&(i.clone(), j.clone())) {
            return None;
        }

        let mut path = vec![i.clone()];
        let mut current = i.clone();

        while current != *j {
            if let Some(next_node) = next.get(&(current.clone(), j.clone())) {
                current = next_node.clone();
                path.push(current.clone());
            } else {
                return None;
            }
        }

        Some(path)
    }

    pub fn get_fields(&mut self) -> (HashMap<NodeId, Node>, HashMap<(NodeId, NodeId), Edge>){
        (self.nodes.clone(), self.edges.clone())
    }
}

#[derive(Eq, PartialEq)]
struct State {
    node: NodeId,
    cost: u32
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

