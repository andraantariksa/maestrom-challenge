
use std::{io, thread};
use std::io::{BufRead, Write};
use std::sync::mpsc;
use std::sync::mpsc::Sender;

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

pub enum Input<REQ, RESP, EV: Send> {
    Request(Message<REQ>),
    Response(Message<RESP>),
    Event(EV),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody<D> {
    pub msg_id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub detail: D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<D> {
    pub src: String,
    pub dest: String,
    pub body: MessageBody<D>,
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

pub trait Node<REQ, RESP, EV>
    where REQ: Serialize + for<'a> Deserialize<'a> + Clone,
          RESP: Serialize + for<'a> Deserialize<'a> + Clone,
          EV: Send {
    fn init<W: Write>(&mut self, writer: &mut W, sender: Sender<Input<REQ, RESP, EV>>, input: &str)
        where Self: Sized {
        let message_req: Message<InitDetails> = serde_json::from_str(input).unwrap();

        if let InitDetails::Init(ref node_init) = message_req.body.detail {
            self.on_init(sender, node_init);
        } else {
            unreachable!();
        }
        message_req.reply(writer, InitDetails::InitOk);
    }

    fn on_init(&mut self, sender: Sender<Input<REQ, RESP, EV>>, message: &NodeInit);

    fn process<W: Write>(&mut self, writer: &mut W, input: Input<REQ, RESP, EV>) {
        match input {
            Input::Request(req) => {
                let detail = self.respond_request(&mut *writer, req.clone());
                req.reply(writer, detail);
            }
            Input::Response(resp) => {
                self.respond_response(&mut *writer, resp)
            }
            Input::Event(ev) => {
                self.respond_event(&mut *writer, ev);
            }
        };
    }

    fn respond_request<W: Write>(&mut self, writer: &mut W, request: Message<REQ>) -> RESP;
    fn respond_response<W: Write>(&mut self, _writer: &mut W, _response: Message<RESP>) {
        unreachable!();
    }
    fn respond_event<W: Write>(&mut self, _writer: &mut W, _event: EV) {
        unreachable!();
    }

    fn get_name(&self) -> &str {
        unreachable!()
    }

    fn send<W: Write>(&self, writer: &mut W, dest: String, detail: REQ) {
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

pub fn process<REQ, RESP, EV>(mut node: impl Node<REQ, RESP, EV>)
    where REQ: Serialize + for<'a> Deserialize<'a> + Clone + Send + 'static,
          RESP: Serialize + for<'a> Deserialize<'a> + Clone + Send + 'static,
          EV: Send + 'static
{
    let mut stdout = io::stdout().lock();

    let (tx, rx) = mpsc::channel();

    {
        let stdin = io::stdin().lock();
        let mut stdin_lines = stdin.lines();
        node.init(&mut stdout, tx.clone(), stdin_lines.next().unwrap().unwrap().as_ref());
    }

    thread::spawn(move || {
        let stdin = io::stdin().lock();
        let stdin_lines = stdin.lines();
        for line in stdin_lines {
            let line = line.as_ref().unwrap();
            if let Ok(message_req) = serde_json::from_str::<Message<REQ>>(line) {
                tx.send(Input::Request(message_req)).unwrap();
            } else if let Ok(message_resp) = serde_json::from_str::<Message<RESP>>(line) {
                tx.send(Input::Response(message_resp)).unwrap();
            }
        }
    });

    for input in rx {
        node.process(&mut stdout, input);
    }
}
