use std::any::{Any, TypeId};

use crate::gcx::GCX;

pub trait NodeManager {
    type NodeBuilder: NodeBuilder;
    type RawNode: Send + Sync;

    fn init(&mut self, _gcx: &GCX) {}

    fn init_node(&mut self, gcx: &GCX, builder: Self::NodeBuilder);
    fn create_node(&mut self) -> Self::RawNode;

    fn update(&mut self);
    fn render(&mut self, _gcx: &GCX) {}
    fn audio_process(&mut self, _buffer: &mut [f32]) {}
}

pub trait AbstractNodeManager {
    fn init(&mut self, gcx: &GCX);
    fn init_node(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>);

    fn create_node(&mut self) -> Box<dyn Any + Send + Sync + 'static>;

    fn update(&mut self);
    fn render(&mut self, gcx: &GCX);
    fn audio_process(&mut self, buffer: &mut [f32]);

    fn ty_id(&self) -> TypeId;
}

impl<T: NodeManager + 'static> AbstractNodeManager for T {
    fn init(&mut self, gcx: &GCX) {
        self.init(gcx)
    }

    fn init_node(&mut self, gcx: &GCX, builder: Box<dyn Any + Send + Sync + 'static>) {
        let builder = Box::<dyn Any + Send + Sync>::downcast::<T::NodeBuilder>(builder).unwrap();
        self.init_node(gcx, *builder);
    }

    fn create_node(&mut self) -> Box<dyn Any + Send + Sync + 'static> {
        Box::new(self.create_node())
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

use crate::scene::SceneTask;

pub trait NodeBuilder: Send + Sync {
    type Node<'a>;
    type NodeManager: NodeManager;

    fn create_node_ref<'a>(
        &self,
        raw: <Self::NodeManager as NodeManager>::RawNode,
        scene: &'a SceneTask,
    ) -> Self::Node<'a>;
}
