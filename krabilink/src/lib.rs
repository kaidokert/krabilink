use element::{Component, PortId};

pub use element::Port;

mod component;
mod connection;
mod element;

mod dynamic;
pub use dynamic::{load_chart, run_simulation};

mod heapless;
