use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

use crate::scene::SceneTask;

pub enum ElementMessage {
    Set(u32, Box<dyn Any + Send + Sync + 'static>),
}

pub trait ElementBuilder: core::fmt::Debug + Send + Sync {
    type ElementRef<'a>;

    fn node_id(&self) -> TypeId;

    fn create_element_ref<'a>(
        &self,
        sender: Sender<ElementMessage>,
        scene: &'a SceneTask,
    ) -> Self::ElementRef<'a>;
}

pub trait AbstractElementBuilder: core::fmt::Debug + Send + Sync {
    fn node_id(&self) -> TypeId;
}
