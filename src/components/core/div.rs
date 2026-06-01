use crate::components::{Component, ComponentBase};

pub struct Div {
    base: ComponentBase,
}

impl Div {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Div {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
