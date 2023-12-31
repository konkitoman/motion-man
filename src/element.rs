use std::any::{Any, TypeId};

use crate::scene::SceneTask;

pub trait NodeBuilder: Send + Sync {
    type Node<'a>;

    fn node_id(&self) -> TypeId;

    fn create_element_ref<'a>(
        &self,
        inner: Box<dyn Any + Send + Sync + 'static>,
        scene: &'a SceneTask,
    ) -> Self::Node<'a>;
}

pub trait AbstractElementBuilder: core::fmt::Debug + Send + Sync {
    fn node_id(&self) -> TypeId;
}
