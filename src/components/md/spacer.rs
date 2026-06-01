use crate::components::{Component, ComponentBase};

pub struct Spacer {
    base: ComponentBase,
}

impl Spacer {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Spacer {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
