use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

use crate::OSend;

#[derive(Debug)]
pub struct Ty {
    pub id: TypeId,
    pub name: &'static str,
}

impl Ty {
    pub fn of<T: 'static>() -> Self {
        let id = TypeId::of::<T>();
        let name = core::any::type_name::<T>();
        Self { id, name }
    }
}

#[derive(Debug)]
pub enum EngineMessage {
    CreateRef(Ty, OSend<Box<dyn Any + Send + Sync + 'static>>),
    CreateNode(Ty, Box<dyn Any + Send + Sync + 'static>),
    Present(OSend<()>),
    Update,
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
