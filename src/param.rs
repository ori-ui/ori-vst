use crate::VstPlugin;

pub trait Param<P: VstPlugin> {
    fn normalize(&self, plugin: &P) -> f64;
}

pub trait Params<P: VstPlugin> {
    fn count(&self) -> usize;

    fn param(&mut self, index: usize) -> Option<&mut dyn Param<P>>;
}

impl<P: VstPlugin> Params<P> for () {
    fn count(&self) -> usize {
        0
    }

    fn param(&mut self, _index: usize) -> Option<&mut dyn Param<P>> {
        None
    }
}
