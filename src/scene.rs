use std::ops::RangeInclusive;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::engine_message::{EngineMessage, Ty};
use crate::node::{NodeBuilder, NodeManager};
use crate::ochannel;
use crate::tween::{Tween, TweenBuilder};
use crate::{engine_message::EngineSender, info::Info};

pub struct SceneTask {
    pub sender: EngineSender,

    pub info: Arc<RwLock<Info>>,
}

impl SceneTask {
    /// this will render how many frames that we say!
    /// if `frames == 0` then will do nothing!
    /// on every frame will the `NodeManager` `render` and `audio_process` will be called!
    pub async fn present(&self, frames: usize) {
        for _ in 0..frames {
            let (send, recv) = ochannel();
            let _ = self.sender.send(EngineMessage::Present(send)).await;
            recv.await.unwrap();
        }
    }

    pub async fn info<O>(&self, reader: impl Fn(&Info) -> O) -> O {
        let info = self.info.read().await;
        reader(&info)
    }

    pub fn fps(&self) -> usize {
        self.info.try_read().unwrap().fps()
    }

    pub fn delta(&self) -> f64 {
        self.info.try_read().unwrap().delta
    }

    /// this will spawn a Node
    pub async fn spawn<T: NodeBuilder + 'static>(&self, builder: T) -> T::Node<'_> {
        let (send, recv) = ochannel();
        self.sender
            .send(EngineMessage::CreateRef(Ty::of::<T::NodeManager>(), send))
            .await;

        let boxed_raw_node = recv.await.unwrap();
        let raw_node = *boxed_raw_node
            .downcast::<<T::NodeManager as NodeManager>::RawNode>()
            .unwrap();

        let element_ref = builder.create_node_ref(raw_node, self);

        self.sender
            .send(EngineMessage::CreateNode(
                Ty::of::<T::NodeManager>(),
                Box::new(builder),
            ))
            .await;

        element_ref
    }

    pub async fn update(&self) {
        self.sender.send(EngineMessage::Update).await;
    }

    pub fn tween<'a>(
        &'a self,
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + 'a + Sync + Send,
    ) -> TweenBuilder<'a> {
        TweenBuilder::new(self, Tween::new(range, time, runner))
    }
}
