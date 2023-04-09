use std::collections::HashMap;
use std::io;
use std::io::{BufRead, Read, Stdin, Stdout, StdoutLock, Write};
use serde_json::Value;
use serde::{Deserialize, Serialize};


#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
#[derive(Debug, Clone, Serialize, Deserialize)]
enum MessageDetail {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo { echo: String },
    EchoOk { echo: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageBody<D> {
    msg_id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    detail: D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message<D> {
    src: String,
    dest: String,
    body: MessageBody<D>,
}

struct Node {
    counter: usize,
}

impl Node {
    fn new() -> Self {
        Self {
            counter: 0
        }
    }

    fn step<W: Write>(&mut self, writer: &mut W, line: String) {
        let message_req: Message<MessageDetail> = serde_json::from_str(&line).unwrap();
        let mut message_resp = message_req.clone();
        message_resp.src = message_req.dest;
        message_resp.dest = message_req.src;
        message_resp.body.msg_id = Some(self.counter);
        message_resp.body.in_reply_to = message_req.body.msg_id;
        message_resp.body.detail = match message_req.body.detail {
            MessageDetail::Init { .. } => {
                self.counter += 1;
                MessageDetail::InitOk
            },
            MessageDetail::Echo { echo } => {
                self.counter += 1;
                MessageDetail::EchoOk { echo }
            },
            MessageDetail::InitOk => unreachable!(),
            MessageDetail::EchoOk { .. } => unreachable!()
        };

        serde_json::to_writer(&mut *writer, &message_resp).unwrap();
        writer.write_all(b"\n").unwrap();
    }
}

fn main_loop() {
    let mut node = Node::new();

    let mut stdin_lines = io::stdin().lock().lines();
    let mut stdout = io::stdout().lock();

    for line in stdin_lines {
        node.step(&mut stdout, line.unwrap());
        stdout.flush().unwrap();
    }
}

fn main() {
    main_loop();
}
