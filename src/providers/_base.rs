use std::sync::mpsc::Sender;

pub trait Provider {
    fn new(sender: Sender<Vec<u8>>) -> Self;
    fn enable(&mut self);
    fn disable(&mut self);
    fn send(&self);
}
