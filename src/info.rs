use std::num::NonZeroU32;

pub struct Info {
    pub delta: f64,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

impl Info {
    pub fn fps(&self) -> usize {
        (1. / self.delta).round() as usize
    }
}
