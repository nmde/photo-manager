# Product

## Register

product

## Users

Hobbyist photographers managing personal libraries of photos and videos. They use this app at their own pace — after a shoot, during an archiving session, or while reviewing a trip — seated at a desk, often in a dim room. They're comfortable with technical tools and expect the app to respect their intelligence, but they're not professionals who demand Lightroom-level density. They want control without friction, depth without clutter.

## Product Purpose

A desktop photo and video management app built on Tauri (Rust backend, Vue frontend). Lets users organize large libraries through an advanced tag system (prerequisite, corequisite, and incompatible tag relationships), geolocation with custom places, layers, and heatmaps, people tagging, a calendar with encrypted journal entries, and RAW format support. The app is the sole tool in the user's workflow — not a companion to something else.

## Brand Personality

Quiet, precise, deliberate. The UI feels like a well-organized darkroom: intentional, calm, unhurried. The reference is Obsidian — dense but organized, power-user focused, dark by default, with structure visible in navigation and graph views rather than decoration. No cheerfulness, no onboarding hand-holding, no engagement-optimized patterns.

## Anti-references

- Generic Material Design: stock Vuetify/Material defaults — green primary, card grids, elevated surfaces with no personality
- Social media apps: infinite scroll discovery feeds, floating action buttons, consumer-soft rounded corners, engagement loops
- Consumer photo apps (Google Photos / iCloud style): pastel accents, overly spacious, designed for non-photographers

## Design Principles

1. **The photo is the product.** The UI serves the image; it never competes with it. When a photo is in view, chrome recedes.
2. **Calm surfaces, not blank ones.** Organized depth over empty minimalism. Structure should be visible, not hidden.
3. **Power is discoverable, not displayed.** Controls exist but don't demand attention. Advanced features reveal themselves through use, not upfront.
4. **Structure over decoration.** Hierarchy through layout, scale, and weight — not color accents, gradients, or ornament.
5. **Dark by nature.** The dark theme is the product's natural state — a darkroom aesthetic, not a trend choice.

## Accessibility & Inclusion

WCAG AA baseline: standard contrast ratios (4.5:1 for body, 3:1 for large text) and full keyboard navigation. No specific reduced-motion requirement, but avoid animations that serve decoration over function.
