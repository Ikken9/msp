use serde::Serialize;
use crate::graph::node::NodeId;

#[derive(Serialize, Clone, Debug)]
pub struct Edge {
    pub source: NodeId,
    pub target: NodeId,
    pub cost: u32,
}

impl Edge {
    pub fn new(source: NodeId, target: NodeId, cost: u32) -> Self {
        Self {
            source,
            target,
            cost
        }
    }
}