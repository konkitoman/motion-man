pub mod color;
pub mod element;
pub mod engine;
pub mod engine_message;
pub mod ffmpeg;
pub mod gcx;
pub mod info;
pub mod node;
pub mod rect;
pub mod scene;
pub mod signal;
pub mod tween;

pub type ORecv<T> = tokio::sync::oneshot::Receiver<T>;
pub type OSend<T> = tokio::sync::oneshot::Sender<T>;
use std::{cell::UnsafeCell, sync::Arc};

pub use tokio::sync::oneshot::channel as ochannel;
pub type SSAny = dyn core::any::Any + core::marker::Sync + core::marker::Send + 'static;

pub struct SCell<T> {
    cell: Arc<UnsafeCell<T>>,
}

impl<T> SCell<T> {
    pub fn set(&self, value: T) {
        unsafe { *self.cell.get() = value };
    }
}

pub struct RCell<T> {
    cell: Arc<UnsafeCell<T>>,
}

unsafe impl<T: Send> Send for RCell<T> {}
unsafe impl<T: Sync> Sync for RCell<T> {}

impl<T> RCell<T> {
    pub fn get(&self) -> &T {
        unsafe { &*self.cell.get() }
    }
}

pub fn create_cell<T>(value: T) -> (SCell<T>, RCell<T>) {
    let cell = Arc::new(UnsafeCell::new(value));
    (SCell { cell: cell.clone() }, RCell { cell })
}
