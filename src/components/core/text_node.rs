use crate::components::{Component, ComponentBase};

pub struct TextNode {
    base: ComponentBase,
}

impl Component for TextNode {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
