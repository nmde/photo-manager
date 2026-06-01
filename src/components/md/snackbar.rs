use crate::components::{Component, ComponentBase};

pub struct Snackbar {
    base: ComponentBase,
}

impl Snackbar {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Snackbar {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
