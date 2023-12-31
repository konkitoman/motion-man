use std::{
    future::Future,
    pin::{pin, Pin},
};

use crate::scene::SceneTask;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct SignalInner<T> {
    sender: Sender<T>,
}

pub struct Signal<'a, T> {
    inner: SignalInner<T>,
    value: T,
    scene: &'a SceneTask,
}

impl<'a, T> Signal<'a, T> {
    pub fn new(inner: SignalInner<T>, scene: &'a SceneTask, value: T) -> Self {
        Self {
            inner,
            scene,
            value,
        }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.value
    }

    pub async fn set(&mut self, value: T)
    where
        T: Clone,
    {
        self.inner.sender.send(value.clone()).await.unwrap();
        self.value = value;
        self.scene.update().await;
    }
}

pub struct Executor<'a> {
    steps: Vec<(
        Box<Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>>,
        Receiver<()>,
        bool,
    )>,
    end: Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> + Send + Sync + 'a>,
    to_wait: Option<Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>>,
    waiting: usize,
}

impl<'a> Executor<'a> {
    pub fn new<
        F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>> + Send + Sync + 'a,
    >(
        end: F,
    ) -> Self {
        Self {
            steps: Vec::new(),
            end: Box::new(end),
            to_wait: None,
            waiting: 0,
        }
    }

    pub fn add<
        F: FnOnce(Sender<()>) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>
            + Send
            + Sync
            + 'a,
    >(
        mut self,
        step: F,
    ) -> Self {
        let (send, recv) = channel(1);
        self.steps.push((Box::new(step(send)), recv, false));
        self
    }
}

impl<'a> Future for Executor<'a> {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(mut to_wait) = self.to_wait.take() {
            if pin!(&mut to_wait).poll(cx).is_pending() {
                self.to_wait = Some(to_wait);
                return std::task::Poll::Pending;
            }
        }

        self.steps.retain_mut(|step| {
            if !step.2 {
                pin!(&mut step.0).poll(cx).is_pending()
            } else {
                true
            }
        });

        let mut count = 0;
        for step in self.steps.iter_mut() {
            if step.1.try_recv().is_ok() {
                count += 1;
                step.2 = true;
            }
        }
        self.waiting += count;

        if self.waiting >= self.steps.len() && !self.steps.is_empty() {
            let mut end = (self.end)();
            self.waiting = 0;
            for step in self.steps.iter_mut() {
                step.2 = false;
            }
            if pin!(&mut end).poll(cx).is_pending() {
                self.to_wait = Some(end);
            }
        }

        if self.steps.is_empty() && self.to_wait.is_none() {
            std::task::Poll::Ready(())
        } else {
            std::task::Poll::Pending
        }
    }
}

pub fn lerp(from: f32, to: f32, time: f64) -> f32 {
    (from as f64 * (1. - time) + to as f64 * time) as f32
}

impl<'a> Signal<'a, [f32; 2]> {
    pub fn tween(&mut self, from: [f32; 2], to: [f32; 2], time: f64) -> Executor {
        let mut sum = 0.;
        Executor::new(|| Box::pin(self.scene.present(1))).add(move |send| {
            Box::pin(async move {
                while sum < 1. {
                    sum += self.scene.delta() / time;
                    let x = lerp(from[0], to[0], sum);
                    let y = lerp(from[1], to[1], sum);
                    self.set([x, y]).await;
                    send.send(()).await.unwrap();
                }
            })
        })
    }
}

pub struct NSignal<T> {
    receiver: Receiver<T>,
}

impl<T> NSignal<T> {
    pub fn get(&mut self) -> Option<T> {
        self.receiver.try_recv().ok()
    }
}

pub fn create_signal<T>() -> (SignalInner<T>, NSignal<T>) {
    let (sender, receiver) = channel(1);
    (SignalInner { sender }, NSignal { receiver })
}
