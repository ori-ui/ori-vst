pub trait EditorHandle: Send + Sync {
    fn quit(&self);

    fn size(&self) -> (u32, u32);

    fn resize(&self, width: u32, height: u32);

    fn resizable(&self) -> bool;

    fn rebuild(&self);
}
