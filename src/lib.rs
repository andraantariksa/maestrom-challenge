use std::collections::HashMap;
use std::io;
use std::io::{BufRead, Read, Stdin, Stdout, StdoutLock, Write};
use serde_json::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInit {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitDetails {
    Init(NodeInit),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageBody<D> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    detail: D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<D> {
    src: String,
    dest: String,
    body: MessageBody<D>,
}

impl<D> Message<D> {
    fn reply<W: Write, R: Serialize>(&self, writer: &mut W, detail: R) {
        let message = Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: MessageBody {
                msg_id: self.body.msg_id.map(|id| id + 1),
                in_reply_to: self.body.msg_id,
                detail,
            },
        };
        serde_json::to_writer(&mut *writer, &message).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

pub trait Node<DREQ: Serialize + for<'a> Deserialize<'a> + Clone, DRESP: Serialize + for<'a> Deserialize<'a> + Clone> {
    fn init<W: Write>(&mut self, writer: &mut W, input: &str)
        where Self: Sized {
        let message_req: Message<InitDetails> = serde_json::from_str(input).unwrap();

        if let InitDetails::Init(ref node_init) = message_req.body.detail {
            self.on_init(node_init);
        } else {
            unreachable!();
        }
        message_req.reply(writer, InitDetails::InitOk);
    }

    fn on_init(&mut self, message: &NodeInit);

    fn step<W: Write>(&mut self, writer: &mut W, input: &str) {
        if let Ok(message_req) = serde_json::from_str::<Message<DREQ>>(input) {
            let detail = self.respond_request(&mut *writer, message_req.body.detail.clone());
            message_req.reply(writer, detail);
        } else if let Ok(message_resp) = serde_json::from_str::<Message<DRESP>>(input) {
            self.respond_response(&mut *writer, message_resp.body.detail.clone());
        }
    }

    fn respond_request<W: Write>(&mut self, writer: &mut W, request_detail: DREQ) -> DRESP;
    fn respond_response<W: Write>(&mut self, writer: &mut W, response_detail: DRESP);

    fn get_name(&self) -> &str {
        unreachable!()
    }

    fn send<W: Write>(&self, writer: &mut W, dest: String, detail: DREQ) {
        let message = Message {
            src: self.get_name().to_owned(),
            dest,
            body: MessageBody {
                msg_id: None,
                in_reply_to: None,
                detail,
            },
        };
        serde_json::to_writer(&mut *writer, &message).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

pub fn process<DETAIL: Serialize + for<'a> Deserialize<'a> + Clone, DETAIL2: Serialize + for<'a> Deserialize<'a> + Clone>(mut node: impl Node<DETAIL, DETAIL2>) {
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();
    let mut stdin_lines = stdin.lines();

    node.init(&mut stdout, stdin_lines.next().unwrap().unwrap().as_ref());

    for line in stdin_lines {
        node.step(&mut stdout, line.unwrap().as_ref());
    }
}
