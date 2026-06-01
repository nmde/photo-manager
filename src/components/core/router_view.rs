use crate::components::{Component, ComponentBase};

pub struct RouterView {
    base: ComponentBase,
    component: Option<Box<dyn Fn(&dyn Component) -> Box<dyn Component>>>,
}

impl RouterView {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(),
            component: None,
        }
    }

    pub fn component<F>(&mut self, slot: F) -> &mut Self
    where
        F: Fn(&dyn Component) -> Box<dyn Component> + 'static,
    {
        self.component = Some(Box::new(slot));
        self
    }
}

impl Component for RouterView {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
