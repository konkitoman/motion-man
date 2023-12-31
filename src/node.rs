use std::any::{Any, TypeId};

use tokio::sync::mpsc::Sender;

use crate::gcx::GCX;

pub trait Node {
    type ElementBuilder;

    fn init(&mut self, _gcx: &GCX) {}

    fn init_element(&mut self, gcx: &GCX, builder: Self::ElementBuilder);
    fn create_element(&mut self) -> Box<dyn Any + Send + Sync + 'static>;

    fn update(&mut self);
    fn render(&mut self, _gcx: &GCX) {}
    fn audio_process(&mut self, _buffer: &mut [f32]) {}
}

pub trait AbstractNode {
    fn init(&mut self, gcx: &GCX);
    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>);

    fn create_element(&mut self) -> Box<dyn Any + Send + Sync + 'static>;

    fn update(&mut self);
    fn render(&mut self, gcx: &GCX);
    fn audio_process(&mut self, buffer: &mut [f32]);

    fn ty_id(&self) -> TypeId;
}

impl<T: Node + 'static> AbstractNode for T {
    fn init(&mut self, gcx: &GCX) {
        self.init(gcx)
    }

    fn init_element(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>) {
        let builder = Box::<dyn Any + Send + Sync>::downcast::<T::ElementBuilder>(builder).unwrap();
        self.init_element(gcx, *builder);
    }

    fn create_element(&mut self) -> Box<dyn Any + Send + Sync + 'static> {
        self.create_element()
    }

    fn update(&mut self) {
        self.update();
    }

    fn render(&mut self, gcx: &GCX) {
        self.render(gcx);
    }

    fn ty_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn audio_process(&mut self, buffer: &mut [f32]) {
        self.audio_process(buffer);
    }
}
