use crate::components::{Component, ComponentBase};

pub struct Layout {
    base: ComponentBase,
}

impl Layout {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Layout {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
