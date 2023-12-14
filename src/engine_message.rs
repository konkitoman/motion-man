use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

use crate::{element::ElementMessage, OSend};

#[derive(Debug)]
pub enum EngineMessage {
    CreateRef(TypeId, OSend<Sender<ElementMessage>>),
    CreateElement(TypeId, Box<dyn Any + Send + Sync + 'static>),
    WaitNextFrame(OSend<()>),
    Submit,
}

pub struct EngineSender {
    pub id: usize,
    pub sender: Sender<(usize, EngineMessage)>,
}

impl EngineSender {
    pub async fn send(&self, msg: EngineMessage) {
        self.sender.send((self.id, msg)).await.unwrap();
    }
}
