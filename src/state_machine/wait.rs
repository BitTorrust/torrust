pub trait Wait {
    fn wait(&mut self);
    fn reset(&mut self);
}
