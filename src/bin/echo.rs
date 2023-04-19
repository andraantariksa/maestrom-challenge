use std::io::Write;
use std::sync::mpsc::Sender;
use maelstorm_challenge::{process, Node, NodeInit, Input, Message};
use serde::{Serialize, Deserialize};


#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RequestDetail {
    Echo { echo: String },
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ResponseDetail {
    EchoOk { echo: String },
}


struct EchoNode {
    name: Option<String>,
}

impl EchoNode {
    fn new() -> Self {
        Self {
            name: None,
        }
    }
}

impl Node<RequestDetail, ResponseDetail, ()> for EchoNode {
    fn on_init(&mut self, _sender: Sender<Input<RequestDetail, ResponseDetail, ()>>, message: &NodeInit) {
        self.name = Some(message.node_id.clone());
    }

    fn respond_request<W: Write>(&mut self, _writer: &mut W, request: Message<RequestDetail>) -> ResponseDetail {
        match request.body.detail {
            RequestDetail::Echo { echo } => {
                ResponseDetail::EchoOk { echo }
            }
        }
    }
}

fn main() {
    process(EchoNode::new());
}
