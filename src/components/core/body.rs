use crate::components::{Component, ComponentBase};

pub struct Body {
    base: ComponentBase,
}

impl Body {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Body {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
