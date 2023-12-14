use std::{future::Future, num::NonZeroU32, pin::Pin, sync::Arc};

use tokio::{
    spawn,
    sync::{
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
    task::JoinHandle,
};

use crate::{
    engine_message::{EngineMessage, EngineSender},
    gcx::GCX,
    info::Info,
    node::AbstractNode,
    scene::SceneTask,
    OSend,
};

pub struct EngineScene {
    id: usize,
    handler: JoinHandle<()>,
}

pub struct Engine {
    scenes: Vec<EngineScene>,

    counter: usize,
    engine_sender: Sender<(usize, EngineMessage)>,
    receiver: Receiver<(usize, EngineMessage)>,

    waiting: Vec<OSend<()>>,

    pub info: Arc<RwLock<Info>>,

    nodes: Vec<Box<dyn AbstractNode>>,
}

impl Engine {
    pub fn new(fps: f64, width: NonZeroU32, height: NonZeroU32) -> Self {
        let info = Info {
            delta: 1. / fps,
            width,
            height,
        };

        let (engine_sender, receiver) = channel(8);

        Self {
            scenes: Vec::default(),
            info: Arc::new(RwLock::new(info)),
            nodes: Vec::default(),
            counter: 0,
            engine_sender,
            receiver,
            waiting: Vec::default(),
        }
    }

    pub fn create_scene(
        &mut self,
        scene_run: impl Fn(SceneTask) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    ) {
        let id = self.counter;
        self.counter += 1;

        let scene = SceneTask {
            sender: EngineSender {
                id,
                sender: self.engine_sender.clone(),
            },

            info: self.info.clone(),
        };

        let engine_scene = EngineScene {
            id,
            handler: spawn(scene_run(scene)),
        };

        self.scenes.push(engine_scene);
    }

    pub fn register_node<T: AbstractNode + Default + 'static>(&mut self) {
        let node = T::default();
        self.nodes.push(Box::new(node));
    }

    pub fn render(&self, gcx: &GCX) {
        for node in self.nodes.iter() {
            node.render(gcx);
        }
    }

    pub fn finished(&self) -> bool {
        self.scenes.is_empty()
    }

    pub fn init(&mut self, gcx: &GCX) {
        for node in self.nodes.iter_mut() {
            node.init(gcx);
        }
    }

    pub async fn run(&mut self, gcx: &GCX) {
        for waiting in self.waiting.drain(..) {
            waiting.send(()).unwrap();
        }

        loop {
            tokio::task::yield_now().await;
            if let Ok((from, msg)) = self.receiver.try_recv() {
                println!("From: {from}, {msg:?}");
                match msg {
                    EngineMessage::WaitNextFrame(send) => {
                        self.waiting.push(send);
                    }
                    EngineMessage::CreateElement(type_id, builder) => {
                        for node in self.nodes.iter_mut() {
                            if node.ty_id() == type_id {
                                node.init_element(gcx, builder);
                                break;
                            }
                        }
                    }
                    EngineMessage::CreateRef(type_id, send) => {
                        for node in self.nodes.iter_mut() {
                            if node.ty_id() == type_id {
                                let s = node.create_ref();
                                send.send(s).unwrap();
                                break;
                            }
                        }
                    }
                    EngineMessage::Submit => {
                        for node in self.nodes.iter_mut() {
                            node.update();
                        }
                    }
                }
            }

            self.scenes
                .retain(|EngineScene { handler, .. }| !handler.is_finished());

            if self.scenes.len() <= self.waiting.len() {
                break;
            }
        }

        println!("Scenes in processing: {}", self.scenes.len());
    }
}
