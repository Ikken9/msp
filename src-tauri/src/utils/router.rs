use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use log::{error, info};
use tauri::State;
use crate::graph::graph::Graph;
use crate::graph::node::NodeId;

pub struct Router {
    pub routes: HashMap<(NodeId, NodeId), (Vec<NodeId>, u32)>
}

impl Router {
    pub fn new() -> Router {
        Router { routes: HashMap::new() }
    }

    pub fn from(routes: HashMap<(NodeId, NodeId), (Vec<NodeId>, u32)>) -> Router {
        Router { routes }
    }

    pub fn route_packet(&self, source: &NodeId, target: &NodeId, state: State<Arc<Mutex<Graph>>>) -> Result<Vec<NodeId>, String> {
        if let Some((initial_path, _cost)) = self.routes.get(&(source.clone(), target.clone())) {
            let mut path = initial_path.clone();
            let mut visited_nodes = HashSet::new();
            let graph = state.lock().unwrap();

            let mut index = 0;
            while index < path.len() {
                let node_id = path[index].clone();
                visited_nodes.insert(node_id.clone());

                if let Some(node) = graph.nodes.get(&node_id) {
                    if !node.available {
                        if index == 0 {
                            return Err(format!("Cannot route packet from {} to {}", source, target));
                        }

                        while index > 0 {
                            let previous_node = &path[index - 1];
                            let mut exclude_nodes = HashSet::new();
                            exclude_nodes.insert(node_id.clone());

                            if let Some(new_subpath) = graph.dijkstra_re_path(previous_node, target, &exclude_nodes) {
                                path.truncate(index);
                                path.extend(new_subpath);

                                if let Some(first_occurrence) = path.iter().position(|node_id| node_id == source) {
                                    info!("Entra al iff");
                                    if let Some(last_occurrence) = path.iter().rposition(|node_id| node_id == source) {
                                        if first_occurrence != last_occurrence {
                                            path.drain(first_occurrence..last_occurrence);
                                        }
                                    }
                                }

                                index -= 1;
                                break;
                            }
                        }

                        if index == 0 {
                            return Err(format!("No alternative path from {} to {} after {}", source, target, node_id));
                        }
                    }
                } else {
                    return Err(format!("Node {} does not exist in the graph", node_id));
                }

                index += 1;
            }

            Ok(path)
        } else {
            Err(format!("No path from {} to {}", source, target))
        }
    }

    pub fn get_shortest_path(&self, source: NodeId, target: NodeId) -> Option<(Vec<NodeId>, u32)> {
        if let Some(path) = self.routes.get(&(source.clone(), target.clone())) {
            return Option::from(path.clone())
        }

        None
    }
}