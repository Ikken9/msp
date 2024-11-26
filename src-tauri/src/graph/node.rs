pub type NodeId = String;

#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub available: bool,
}

impl Node {
    pub fn new(id: NodeId) -> Self {
        Self { id , available: true }
    }
}