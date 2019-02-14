use crate::node::{Index, Node, NodeBase, NodeId};

mod create;

#[derive(Clone, Copy, Debug)]
pub struct NodeTemplate {
    pub nw: NodeId,
    pub ne: NodeId,
    pub sw: NodeId,
    pub se: NodeId,
}

#[derive(Clone, Debug)]
pub struct Store {
    ids: hashbrown::HashMap<NodeBase, NodeId>,
    nodes: Vec<Node>,
    steps: Vec<Option<NodeId>>,
    jumps: Vec<Option<NodeId>>,
    step_log_2: u8,
}

impl Store {
    pub fn new() -> Self {
        Self {
            ids: hashbrown::HashMap::default(),
            nodes: vec![],
            steps: vec![],
            jumps: vec![],
            step_log_2: 0,
        }
    }

    pub fn get(&self, id: NodeId) -> &Node {
        &self.nodes[id.index.0 as usize]
    }

    pub fn step_log_2(&self) -> u8 {
        self.step_log_2
    }

    pub fn set_step_log_2(&mut self, step_log_2: u8) {
        if self.step_log_2 != step_log_2 {
            self.step_log_2 = step_log_2;
            self.steps = vec![None; self.steps.len()];
        }
    }

    pub fn add_jump(&mut self, id: NodeId, jump: NodeId) {
        self.jumps[id.index.0 as usize] = Some(jump);
    }

    pub fn get_jump(&self, id: NodeId) -> Option<NodeId> {
        self.jumps[id.index.0 as usize]
    }

    pub fn add_step(&mut self, id: NodeId, step: NodeId) {
        self.steps[id.index.0 as usize] = Some(step);
    }

    pub fn get_step(&self, id: NodeId) -> Option<NodeId> {
        self.steps[id.index.0 as usize]
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        if let Some(&id) = self.ids.get(&node.base) {
            id
        } else {
            let id = NodeId {
                index: Index(self.nodes.len() as u32),
            };
            self.ids.insert(node.base, id);
            self.nodes.push(node);
            self.steps.push(None);
            self.jumps.push(None);
            id
        }
    }
}
