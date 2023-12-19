use std::any::{Any, TypeId};

use crate::scene::SceneTask;

pub trait ElementBuilder: core::fmt::Debug + Send + Sync {
    type Element<'a>;

    fn node_id(&self) -> TypeId;

    fn create_element_ref<'a>(
        &self,
        inner: Box<dyn Any + Send + Sync + 'static>,
        scene: &'a SceneTask,
    ) -> Self::Element<'a>;
}

pub trait AbstractElementBuilder: core::fmt::Debug + Send + Sync {
    fn node_id(&self) -> TypeId;
}
