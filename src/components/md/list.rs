use crate::{
    components::{Component, ComponentBase, md::icon::Icon},
    styles::color::Color,
};

pub struct List {
    base: ComponentBase,
    color: Option<Color>,
    nav: Option<bool>,
}

impl List {
    pub fn new(children: Vec<&dyn Component>) -> Self {
        Self {
            base: ComponentBase::new(),
            color: None,
            nav: None,
        }
    }

    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Some(color);
        self
    }

    pub fn nav(&mut self, value: bool) -> &mut Self {
        self.nav = Some(value);
        self
    }
}

impl Component for List {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}

pub struct ListItem<T> {
    base: ComponentBase,
    prepend_icon: Option<Icon>,
    title: Option<String>,
    to: Option<T>,
}

impl<T> ListItem<T> {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(),
            prepend_icon: None,
            title: None,
            to: None,
        }
    }

    pub fn prepend_icon(&mut self, icon: Icon) -> &mut Self {
        self.prepend_icon = Some(icon);
        self
    }

    pub fn title(&mut self, title: &str) -> &mut Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn to(&mut self, to: T) -> &mut Self {
        self.to = Some(to);
        self
    }
}

impl<T> Component for ListItem<T> {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}

pub struct Divider {
    base: ComponentBase,
}

impl Divider {
    pub fn new() -> Self {
        Self {
            base: ComponentBase::new(),
        }
    }
}

impl Component for Divider {
    fn base(&self) -> &ComponentBase {
        &self.base
    }

    fn base_mut(&mut self) -> &mut ComponentBase {
        &mut self.base
    }
}
