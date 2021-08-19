use std::fmt;

use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub struct Message {
    pub text: String,
    ack_tx: oneshot::Sender<()>,
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}

pub struct MessageSender {
    tx: mpsc::Sender::<Message>,
}

pub struct MessageReceiver {
    rx: mpsc::Receiver::<Message>,
}

pub fn new(buffer: usize) -> (MessageSender, MessageReceiver) {
    let (tx, mut rx) = mpsc::channel::<Message>(buffer);
    (
        MessageSender{tx},
        MessageReceiver{rx},
    )
}

impl Message {
    pub fn ack(self) -> Result<(), ()> {
        self.ack_tx.send(())
    }
}

impl MessageSender {
    pub async fn send_and_wait(&self, text: String) {
        let (ack_tx, mut ack_rx) = oneshot::channel::<()>();
        let msg = Message{text, ack_tx};
        self.tx.send(msg).await.unwrap();
        ack_rx.await.unwrap();
    }
}

impl MessageReceiver {
    pub async fn recv(&mut self) -> Option<Message> {
        self.rx.recv().await
    }
}
