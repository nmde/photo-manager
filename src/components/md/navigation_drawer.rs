use crate::components::{Component, ComponentBase};

pub struct NavigationDrawer {
    base: ComponentBase,
    expand_on_hover: Option<bool>,
    permanent: Option<bool>,
    rail: Option<bool>,
}

impl NavigationDrawer {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
            expand_on_hover: None,
            permanent: None,
            rail: None,
        }
    }

    pub fn expand_on_hover(&mut self, value: bool) -> &mut Self {
        self.expand_on_hover = Some(value);
        self
    }

    pub fn permanent(&mut self, value: bool) -> &mut Self {
        self.permanent = Some(value);
        self
    }

    pub fn rail(&mut self, value: bool) -> &mut Self {
        self.rail = Some(value);
        self
    }
}

impl Component for NavigationDrawer {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
