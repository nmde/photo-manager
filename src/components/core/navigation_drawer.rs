use crate::components::{Component, ComponentBase};

pub struct NavigationDrawer {
    base: ComponentBase,
}

impl Component for NavigationDrawer {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
