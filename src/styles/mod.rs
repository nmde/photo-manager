use crate::styles::{color::Color, measurement::Measurement};

pub mod color;
pub mod measurement;
pub mod timing;
pub mod tokens;

#[derive(Clone)]
pub struct Style {
    pub background_color: Option<Color>,
    pub height: Option<Measurement>,
    pub width: Option<Measurement>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            background_color: None,
            height: None,
            width: None,
        }
    }

    pub fn background_color(&mut self, color: Color) -> &mut Self {
        self.background_color = Some(color);
        self
    }

    pub fn height(&mut self, height: Measurement) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn width(&mut self, width: Measurement) -> &mut Self {
        self.width = Some(width);
        self
    }

    /// Overrides this style with the given style
    /// For each style property, if the other style defines a Some value, it will override this style's property.
    pub fn merge(&self, other: &Style) -> Style {
        let mut merged = self.clone();
        if other.background_color.is_some() {
            merged.background_color = other.background_color;
        }
        if other.height.is_some() {
            merged.height = other.height;
        }
        if other.width.is_some() {
            merged.width = other.width;
        }
        merged
    }
}

/*
html {
  overflow: hidden;
}

body {
  font-variant-numeric: tabular-nums;
  letter-spacing: var(--text-tracking-body);
  font-kerning: normal;
}

.v-toolbar-title__placeholder {
  font-weight: var(--text-weight-semibold);
  letter-spacing: var(--text-tracking-tight);
}

.v-navigation-drawer .v-list-item-title {
  font-weight: var(--text-weight-medium);
  letter-spacing: var(--text-tracking-body);
}

.v-chip .v-chip__content {
  letter-spacing: var(--text-tracking-label);
}

/* Route crossfade */
.route-enter-active {
  transition: opacity var(--duration-standard) var(--ease-out-expo);
}
.route-leave-active {
  transition: opacity 80ms var(--ease-standard);
}
.route-enter-from,
.route-leave-to {
  opacity: 0;
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}

.toolbar-controls {
  display: flex;
  height: 100%;
  width: -webkit-fill-available;
  align-items: center;
}

.toolbar-controls > * {
  padding-top: 20px;
}

.fill-height {
  height: 100%;
}

.tag-graph-node {
  stroke-width: 1;
}

.tag-graph-link {
  stroke-width: 1;
}

.tag-graph-link--default {
  stroke: var(--color-surface-raised);
}

.tag-graph-node--default {
  fill: var(--color-surface-raised);
  stroke: var(--color-surface-raised);
}

.tag-graph-label {
  font-size: 10px;
  stroke: var(--color-background);
  paint-order: stroke;
  cursor: pointer;
}

.tag-graph-label--default {
  fill: var(--color-text-primary);
}
*/
