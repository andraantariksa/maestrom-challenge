use std::collections::{HashMap, HashSet};
use maelstorm_challenge::{Node, NodeInit, process};
use std::io::Write;
use serde::{Deserialize, Serialize};

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RequestDetail {
    Topology { topology: HashMap<String, Vec<String>> },
    Read,
    Broadcast { message: usize },
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ResponseDetail {
    TopologyOk,
    ReadOk { messages: HashSet<usize> },
    BroadcastOk,
}

struct BroadcastNode {
    name: Option<String>,
    ids: HashSet<usize>,
    topology: Option<HashMap<String, Vec<String>>>,
}

impl BroadcastNode {
    fn new() -> Self {
        Self {
            name: None,
            ids: HashSet::new(),
            topology: None,
        }
    }
}

impl Node<RequestDetail, ResponseDetail> for BroadcastNode {
    fn on_init(&mut self, message: &NodeInit) {
        self.name = Some(message.node_id.clone());
    }

    fn respond_request<W: Write>(&mut self, writer: &mut W, request_detail: RequestDetail) -> ResponseDetail {
        match request_detail {
            RequestDetail::Topology { topology } => {
                self.topology = Some(topology);
                ResponseDetail::TopologyOk
            }
            RequestDetail::Read => {
                ResponseDetail::ReadOk {
                    messages: self.ids.clone()
                }
            }
            RequestDetail::Broadcast { message } => {
                if self.ids.contains(&message) {
                    return ResponseDetail::BroadcastOk;
                }

                if let Some(ref topology) = self.topology {
                    let node_connections = &topology[self.name.as_ref().unwrap()];
                    for node in node_connections {
                        self.send(writer, node.to_owned(), RequestDetail::Broadcast { message });
                    }
                }

                self.ids.insert(message);
                ResponseDetail::BroadcastOk
            }
        }
    }

    fn respond_response<W: Write>(&mut self, writer: &mut W, response_detail: ResponseDetail) {
        match response_detail {
            ResponseDetail::TopologyOk => unreachable!(),
            ResponseDetail::ReadOk { .. } => unreachable!(),
            ResponseDetail::BroadcastOk => {},
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_ref().unwrap()
    }
}

fn main() {
    process(BroadcastNode::new());
}

