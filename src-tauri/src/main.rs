// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use log::info;
use crate::graph::graph::Graph;

use crate::invoker::{add_edge, add_node, get_graph, remove_node, remove_edge, get_shortest_path, set_node_availability, route_packet};

use tauri_plugin_log::{LogTarget};
use crate::utils::router::Router;

mod graph;
mod utils;
mod invoker;

fn main() {
    let graph = Arc::new(Mutex::new(Graph::new()));

    graph.lock().unwrap().add_edge("A".to_string(), "B".to_string(), 100).unwrap();
    graph.lock().unwrap().add_edge("A".to_string(), "C".to_string(), 56).unwrap();
    graph.lock().unwrap().add_edge("B".to_string(), "E".to_string(), 14).unwrap();
    graph.lock().unwrap().add_edge("C".to_string(), "D".to_string(), 77).unwrap();
    graph.lock().unwrap().add_edge("E".to_string(), "F".to_string(), 56).unwrap();
    graph.lock().unwrap().add_edge("E".to_string(), "G".to_string(), 75).unwrap();
    graph.lock().unwrap().add_edge("G".to_string(), "C".to_string(), 86).unwrap();
    graph.lock().unwrap().add_edge("H".to_string(), "C".to_string(), 81).unwrap();
    graph.lock().unwrap().add_edge("H".to_string(), "F".to_string(), 14).unwrap();
    graph.lock().unwrap().add_edge("F".to_string(), "C".to_string(), 76).unwrap();
    graph.lock().unwrap().add_edge("F".to_string(), "A".to_string(), 66).unwrap();
    graph.lock().unwrap().add_edge("F".to_string(), "B".to_string(), 71).unwrap();
    graph.lock().unwrap().add_edge("F".to_string(), "D".to_string(), 76).unwrap();
    graph.lock().unwrap().add_edge("E".to_string(), "J".to_string(), 92).unwrap();
    graph.lock().unwrap().add_edge("E".to_string(), "K".to_string(), 81).unwrap();
    graph.lock().unwrap().add_edge("A".to_string(), "K".to_string(), 12).unwrap();
    graph.lock().unwrap().add_edge("K".to_string(), "I".to_string(), 76).unwrap();
    graph.lock().unwrap().add_edge("I".to_string(), "D".to_string(), 15).unwrap();
    graph.lock().unwrap().add_edge("D".to_string(), "L".to_string(), 55).unwrap();
    graph.lock().unwrap().add_edge("L".to_string(), "C".to_string(), 64).unwrap();

    let routes = graph.lock().unwrap().floyd_warshall_map().clone();
    let router = Arc::new(Mutex::new(Router::from(routes)));

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().targets([
            LogTarget::LogDir,
            LogTarget::Stderr,
            LogTarget::Stdout,
            LogTarget::Webview,
        ]).build())
        .manage(graph)
        .manage(router)
        .invoke_handler(tauri::generate_handler![
            get_graph,
            add_node,
            add_edge,
            remove_node,
            remove_edge,
            get_shortest_path,
            set_node_availability,
            route_packet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}