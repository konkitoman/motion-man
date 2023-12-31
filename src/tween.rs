use std::{
    future::Future,
    ops::RangeInclusive,
    pin::{pin, Pin},
    task::Poll,
};

use crate::scene::SceneTask;

pub struct Tween<'a> {
    range: RangeInclusive<f32>,
    time: f32,
    runner: Box<dyn FnMut(f32) + Send + Sync + 'a>,
    x: f32,
}

impl<'a> Tween<'a> {
    pub fn new(
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + Send + Sync + 'a,
    ) -> Self {
        Self {
            x: *range.start(),
            range,
            time,
            runner: Box::new(runner),
        }
    }
}

pub enum TweenBuilderStage<'a> {
    Init {
        task: &'a SceneTask,
        tweens: Vec<Tween<'a>>,
    },
    Running(Pin<Box<dyn Future<Output = ()> + Send + Sync + 'a>>),
}

pub struct TweenBuilder<'a> {
    stage: Option<TweenBuilderStage<'a>>,
}

impl<'a> TweenBuilder<'a> {
    pub fn new(task: &'a SceneTask, tween: Tween<'a>) -> Self {
        Self {
            stage: Some(TweenBuilderStage::Init {
                task,
                tweens: vec![tween],
            }),
        }
    }

    pub fn tween(
        mut self,
        range: RangeInclusive<f32>,
        time: f32,
        runner: impl FnMut(f32) + Sync + Send + 'a,
    ) -> Self {
        if let TweenBuilderStage::Init { tweens, .. } = self.stage.as_mut().unwrap() {
            tweens.push(Tween::new(range, time, runner));
        }
        self
    }
}

impl<'a> Future for TweenBuilder<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let task = self.stage.take().unwrap();

        match task {
            TweenBuilderStage::Init { task, mut tweens } => {
                let future = Box::pin(async move {
                    let delta = task.info(|i| i.delta).await as f32;
                    loop {
                        tweens.retain_mut(|tween| {
                            let start = *tween.range.start();
                            let end = *tween.range.end();

                            let inverse = start > end;
                            if inverse {
                                tween.x -= (delta / tween.time) * (start - end);
                                (tween.runner)(tween.x);

                                tween.x >= end
                            } else {
                                tween.x += (delta / tween.time) * (end - start);
                                (tween.runner)(tween.x);

                                tween.x <= end
                            }
                        });
                        task.update().await;
                        task.present(1).await;

                        if tweens.is_empty() {
                            break;
                        }
                    }
                });

                self.stage.replace(TweenBuilderStage::Running(future));
            }
            TweenBuilderStage::Running(mut run) => {
                let res = pin!(&mut run).poll(cx);
                self.stage.replace(TweenBuilderStage::Running(run));
                return res;
            }
        }

        if let TweenBuilderStage::Running(mut run) = self.stage.take().unwrap() {
            let res = pin!(&mut run).poll(cx);
            self.stage.replace(TweenBuilderStage::Running(run));
            res
        } else {
            Poll::Ready(())
        }
    }
}
