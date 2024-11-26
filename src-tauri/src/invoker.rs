use std::collections::{HashMap, HashSet};
use serde::Serialize;
use std::sync::{Arc, Mutex};
use log::info;
use tauri::State;
use crate::graph::edge::Edge;
use crate::graph::graph::Graph;
use crate::graph::node::NodeId;
use crate::utils::router::Router;

#[derive(Serialize, Clone)]
pub struct GraphState {
    nodes: HashSet<NodeId>,
    edges: HashMap<(NodeId, NodeId), Edge>,
}

#[derive(Serialize)]
pub struct SerializableGraphState {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Serialize)]
pub struct GraphEdge {
    source: String,
    target: String,
    cost: u32,
}

#[derive(Serialize)]
pub struct GraphNode {
    id: String,
    availability: bool,
}

#[derive(Serialize)]
pub struct SerializablePath {
    path: Vec<NodeId>,
    cost: u32
}

#[tauri::command]
pub fn get_graph(state: State<Arc<Mutex<Graph>>>) -> SerializableGraphState {
    let mut graph = state.lock().unwrap();

    let (nodes_field, edges_field) = graph.get_fields();

    let nodes: Vec<GraphNode> = nodes_field
        .into_iter()
        .map(|(node_id, node)| GraphNode {
            id: node_id.to_string(),
            availability: node.available
        })
        .collect();

    let edges: Vec<GraphEdge> = edges_field
        .into_iter()
        .map(|((source, target), edge)| GraphEdge {
            source: source.to_string(),
            target: target.to_string(),
            cost: edge.cost,
        })
        .collect();

    SerializableGraphState { nodes, edges }
}

#[tauri::command]
pub fn add_node(state: State<Arc<Mutex<Graph>>>, id: NodeId) -> Result<(), String> {
    let mut graph = state.lock().unwrap();
    graph.add_node(&id)
}

#[tauri::command]
pub fn add_edge(router_state: State<Arc<Mutex<Router>>>, graph_state: State<Arc<Mutex<Graph>>>, source: NodeId, target: NodeId, cost: u32) -> Result<(), String> {
    let mut graph = graph_state.lock().unwrap();
    graph.add_edge(source, target, cost).expect("Failed to add edge");

    let mut router = router_state.lock().unwrap();
    router.routes = graph.floyd_warshall_map();
    Ok(())
}

#[tauri::command]
pub fn remove_node(state: State<Arc<Mutex<Graph>>>, id: NodeId) -> Result<(), String> {
    let mut graph = state.lock().unwrap();
    graph.remove_node(&id)
}

#[tauri::command]
pub fn remove_edge(state: State<Arc<Mutex<Graph>>>, source: NodeId, target: NodeId) -> Result<(), String> {
    let mut graph = state.lock().unwrap();
    graph.remove_edge(source, target)
}

#[tauri::command]
pub fn set_node_availability(state: State<Arc<Mutex<Graph>>>, id: NodeId, available: bool) -> Result<(), String> {
    let mut graph = state.lock().unwrap();
    graph.set_node_availability(&*id, available)
}

#[tauri::command]
pub fn get_shortest_path(state: State<Arc<Mutex<Router>>>, start: NodeId, target: NodeId) -> SerializablePath {
    let router = state.lock().unwrap();
    if let Some(shortest_path) = router.get_shortest_path(start, target) {
        let (path, cost) = shortest_path;
        return SerializablePath { path, cost }
    }

    SerializablePath { path: vec![], cost: 0 }
}

#[tauri::command]
pub fn route_packet(router_state: State<Arc<Mutex<Router>>>, graph_state: State<Arc<Mutex<Graph>>>, start: NodeId, target: NodeId) -> SerializablePath {
    let router = router_state.lock().unwrap();

    if let Ok(path) = router.route_packet_v2(&start, &target, graph_state) {
        return SerializablePath { path, cost: 0 }
    }

    SerializablePath { path: vec![], cost: 0 }
}