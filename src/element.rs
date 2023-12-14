use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

pub enum ElementMessage {
    Set(u32, Box<dyn Any + Send + Sync + 'static>),
}

pub trait ElementBuilder: core::fmt::Debug + Send + Sync {
    type ElementRef;

    fn node_id(&self) -> TypeId;

    fn create_element_ref(&self, sender: Sender<ElementMessage>) -> Self::ElementRef;
}

pub trait AbstractElementBuilder: core::fmt::Debug + Send + Sync {
    fn node_id(&self) -> TypeId;
}
