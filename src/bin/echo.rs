use std::io::Write;
use maelstorm_challenge::{process, Node, NodeInit};
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

impl Node<RequestDetail, ResponseDetail> for EchoNode {
    fn on_init(&mut self, message: &NodeInit) {
        self.name = Some(message.node_id.clone());
    }

    fn respond_request<W: Write>(&mut self, writer: &mut W, input: RequestDetail) -> ResponseDetail {
        match input {
            RequestDetail::Echo { echo } => {
                ResponseDetail::EchoOk { echo }
            }
        }
    }

    fn respond_response<W: Write>(&mut self, writer: &mut W, response_detail: ResponseDetail) {
        unreachable!()
    }
}

fn main() {
    process(EchoNode::new());
}
