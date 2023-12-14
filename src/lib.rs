pub mod color;
pub mod element;
pub mod engine;
pub mod engine_message;
pub mod gcx;
pub mod info;
pub mod node;
pub mod rect;
pub mod scene;
pub mod tween;

pub type ORecv<T> = tokio::sync::oneshot::Receiver<T>;
pub type OSend<T> = tokio::sync::oneshot::Sender<T>;
pub use tokio::sync::oneshot::channel as ochannel;
