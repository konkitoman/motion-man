use std::ops::RangeInclusive;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::element::ElementBuilder;
use crate::engine_message::EngineMessage;
use crate::ochannel;
use crate::tween::{Tween, TweenBuilder};
use crate::{engine_message::EngineSender, info::Info};

pub struct SceneTask {
    pub sender: EngineSender,

    pub info: Arc<RwLock<Info>>,
}

impl SceneTask {
    pub async fn wait(&self, frames: usize) {
        for _ in 0..frames {
            let (send, recv) = ochannel();
            let _ = self.sender.send(EngineMessage::WaitNextFrame(send)).await;
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

    pub async fn spawn_element<'a, T: ElementBuilder + 'static>(
        &'a self,
        builder: T,
    ) -> T::Element<'a> {
        let (send, recv) = ochannel();
        self.sender
            .send(EngineMessage::CreateRef(builder.node_id(), send))
            .await;

        let element_ref = builder.create_element_ref(recv.await.unwrap(), self);

        self.sender
            .send(EngineMessage::CreateElement(
                builder.node_id(),
                Box::new(builder),
            ))
            .await;

        element_ref
    }

    pub async fn update(&self) {
        self.sender.send(EngineMessage::Submit).await;
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
