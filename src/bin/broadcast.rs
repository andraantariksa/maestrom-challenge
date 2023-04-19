use std::collections::{HashMap, HashSet};
use maelstorm_challenge::{Input, Message, Node, NodeInit, process};
use std::io::Write;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RequestDetail {
    Topology { topology: HashMap<String, HashSet<String>> },
    Read,
    Broadcast { message: usize },
    Gossip { ids: HashSet<usize> },
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ResponseDetail {
    TopologyOk,
    ReadOk { messages: HashSet<usize> },
    BroadcastOk,
    GossipOk { ids: HashSet<usize> },
}

enum Event {
    Sync
}

struct BroadcastNode {
    name: Option<String>,
    ids: HashSet<usize>,
    topology: Option<HashMap<String, HashSet<String>>>,
    known_neighbor_value: HashMap<String, HashSet<usize>>,
}

impl BroadcastNode {
    fn new() -> Self {
        Self {
            name: None,
            ids: HashSet::new(),
            topology: None,
            known_neighbor_value: HashMap::new(),
        }
    }
}

impl Node<RequestDetail, ResponseDetail, Event> for BroadcastNode {
    fn on_init(&mut self, sender: Sender<Input<RequestDetail, ResponseDetail, Event>>, message: &NodeInit) {
        self.name = Some(message.node_id.clone());

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(300));
                sender.send(Input::Event(Event::Sync)).unwrap();
            }
        });
    }

    fn respond_request<W: Write>(&mut self, _writer: &mut W, request: Message<RequestDetail>) -> ResponseDetail {
        match request.body.detail {
            RequestDetail::Topology { topology } => {
                // Populating reachable nodes within topology
                let mut nodes = topology.keys().map(|x| x.to_owned()).collect::<HashSet<String>>();
                for (_, connected_nodes) in &topology {
                    nodes.extend(connected_nodes.to_owned());
                }
                self.known_neighbor_value.extend(nodes.into_iter().map(|node| (node, HashSet::new())));

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

                self.ids.insert(message);
                ResponseDetail::BroadcastOk
            }
            RequestDetail::Gossip { ids } => {
                // We know that the requesting node has `ids` values
                self.known_neighbor_value.get_mut(&request.src).unwrap().extend(ids.clone());
                self.ids.extend(ids);
                // Let the requesting node to know all of our values
                ResponseDetail::GossipOk { ids: self.ids.clone() }
            }
        }
    }

    fn respond_response<W: Write>(&mut self, _writer: &mut W, response: Message<ResponseDetail>) {
        match response.body.detail {
            ResponseDetail::TopologyOk => unreachable!(),
            ResponseDetail::ReadOk { .. } => unreachable!(),
            ResponseDetail::BroadcastOk => unreachable!(),
            // Keep our values
            ResponseDetail::GossipOk { ids } => {
                self.ids.extend(ids);
            }
        }
    }

    fn respond_event<W: Write>(&mut self, writer: &mut W, event: Event) {
        match event {
            Event::Sync => {
                if let Some(ref topology) = self.topology {
                    let default = HashSet::new();
                    let node_connections = &topology[self.name.as_ref().unwrap()];
                    for node in node_connections {
                        // Exclude values known by node we're requesting to
                        let known_node_values = self.known_neighbor_value.get(node).unwrap_or(&default);
                        let sent_ids = self.ids.clone().difference(known_node_values).copied().collect::<HashSet<usize>>();

                        if sent_ids.is_empty() {
                            continue;
                        }

                        self.send(writer, node.to_owned(), RequestDetail::Gossip { ids: sent_ids });
                    }
                }
            }
        };
    }

    fn get_name(&self) -> &str {
        self.name.as_ref().unwrap()
    }
}

// This is basically just a 2 general problem
// We should never assume that another node knows what we know, unless they had confirm it
// So we send them everything we know until they sent us back everything what they know, then
// we can assume that they know what we're sending from the former event
fn main() {
    process(BroadcastNode::new());
}

