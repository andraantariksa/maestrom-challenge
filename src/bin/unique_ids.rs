use maelstorm_challenge::{Node, NodeInit, process};
use std::io::Write;
use serde::{Deserialize, Serialize};

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum RequestDetail {
    Generate,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ResponseDetail {
    GenerateOk { id: String },
}

struct UniqueIdsNode {
    name: Option<String>,
    id_count: usize,
}

impl UniqueIdsNode {
    fn new() -> Self {
        Self {
            name: None,
            id_count: 0,
        }
    }
}

impl Node<RequestDetail, ResponseDetail> for UniqueIdsNode {
    fn on_init(&mut self, message: &NodeInit) {
        self.name = Some(message.node_id.clone());
    }

    fn respond_request<W: Write>(&mut self, writer: &mut W, input: RequestDetail) -> ResponseDetail {
        match input {
            RequestDetail::Generate => {
                let mut id = String::new();
                if let Some(name) = &self.name {
                    id.push_str(name);
                    id.push('-');
                };
                id = id + &*self.id_count.to_string();
                self.id_count += 1;
                ResponseDetail::GenerateOk { id }
            }
        }
    }

    fn respond_response<W: Write>(&mut self, writer: &mut W, response_detail: ResponseDetail) {
        unreachable!()
    }
}

fn main() {
    process(UniqueIdsNode::new());
}
