pub trait Provider {
    fn start(&self);
    fn stop(&self);
}
