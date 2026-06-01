use crate::components::{Component, ComponentBase};

pub struct Main {
    base: ComponentBase,
}

impl Main {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Main {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
