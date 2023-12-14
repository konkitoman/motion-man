use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

use crate::{element::ElementMessage, gcx::GCX};

pub trait Node {
    type ElementBuilder;

    fn init(&mut self, gcx: &GCX);

    fn init_element(&mut self, gcx: &GCX, builder: Self::ElementBuilder);
    fn create_ref(&mut self) -> Sender<ElementMessage>;

    fn update(&mut self);
    fn render(&self, gcx: &GCX);
}

pub trait AbstractNode: core::fmt::Debug {
    fn init(&mut self, gcx: &GCX);
    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>);

    fn create_ref(&mut self) -> Sender<ElementMessage>;

    fn update(&mut self);
    fn render(&self, gcx: &GCX);

    fn ty_id(&self) -> TypeId;
}

impl<T: Node + core::fmt::Debug + 'static> AbstractNode for T {
    fn init(&mut self, gcx: &GCX) {
        self.init(gcx)
    }

    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>) {
        let builder = Box::<dyn Any + Send + Sync>::downcast::<T::ElementBuilder>(builder).unwrap();
        self.init_element(gcx, *builder);
    }

    fn create_ref(&mut self) -> Sender<ElementMessage> {
        self.create_ref()
    }

    fn update(&mut self) {
        self.update();
    }

    fn render(&self, gcx: &GCX) {
        self.render(gcx);
    }

    fn ty_id(&self) -> TypeId {
        TypeId::of::<T>()
    }
}