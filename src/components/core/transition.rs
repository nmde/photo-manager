use crate::components::{Component, ComponentBase};

pub enum TransitionMode {
    OutIn,
}

pub struct Transition {
    base: ComponentBase,
    mode: Option<TransitionMode>,
}

impl Transition {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
            mode: None,
        }
    }

    pub fn mode(&mut self, mode: TransitionMode) -> &mut Self {
        self.mode = Some(mode);
        self
    }
}

impl Component for Transition {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
