---
name: Photo Manager
description: Desktop photo and video management for hobbyist photographers. The Observatory.
colors:
  primary: "oklch(63% 0.09 245)"
  background: "oklch(9% 0.003 245)"
  surface: "oklch(14% 0.003 245)"
  surface-raised: "oklch(18% 0.003 245)"
  text-primary: "oklch(91% 0.005 245)"
  text-secondary: "oklch(63% 0.005 245)"
  error: "oklch(58% 0.13 14)"
typography:
  headline:
    fontFamily: "Roboto, sans-serif"
    fontSize: "20px"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "normal"
  title:
    fontFamily: "Roboto, sans-serif"
    fontSize: "16px"
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: "normal"
  body:
    fontFamily: "Roboto, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "0.015em"
  label:
    fontFamily: "Roboto, sans-serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: "0.04em"
rounded:
  sm: "4px"
  md: "8px"
  pill: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "16px"
  lg: "24px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.background}"
    rounded: "{rounded.sm}"
    padding: "8px 20px"
  button-primary-hover:
    backgroundColor: "oklch(70% 0.09 245)"
    textColor: "{colors.background}"
    rounded: "{rounded.sm}"
    padding: "8px 20px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.sm}"
    padding: "8px 20px"
  chip-tag:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.pill}"
    padding: "2px 10px"
  nav-item-active:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.background}"
    rounded: "{rounded.md}"
    padding: "8px 12px"
---

# Design System: Photo Manager

## 1. Overview

**Creative North Star: "The Observatory"**

The interface exists to serve the photograph. Like an observatory dome, everything in the environment is dark by design — not as aesthetic statement, but because the work demands it. The canvas recedes so what you're examining stands forward. Controls are present but never intrusive: a rail of icons at the periphery, a toolbar holding only what the current task requires, a detail panel that opens when called and collapses when dismissed.

The design vocabulary is minimal and systematic. No decorative surfaces. No color used for atmosphere. Hierarchy comes from position and weight: the photo grid is the center of mass, and the chrome around it compresses toward the edges. When a photo is open in the detail pane, the grid shrinks to a column — the UI physically yields.

This system rejects the consumer warmth of Google Photos, the busy panel density of Adobe's tools, and the personality-free legibility of stock Material Design. It is a single-surface desktop tool built for someone who knows what they're doing and wants the interface to respect that.

**Key Characteristics:**
- Dark-first: the background is the canvas, not a style choice
- Functional restraint: every element has exactly one job
- Quiet hierarchy: structure through layout and weight, never color saturation
- The photo leads: chrome compresses when content is active
- Keyboard-reachable: all primary actions accessible without a mouse

## 2. Colors: The Observatory Palette

A near-monochrome dark system with one muted blue accent. Color is not atmosphere here — it is signal.

### Primary

- **Observatory Slate** (`oklch(63% 0.09 245)`, approx `#5f87ac`): The single accent. Used on active navigation items, primary buttons, progress indicators, and focused input borders. Muted enough to not demand attention; present enough to mark what's active. Used on under 15% of any given screen. Its restraint is the point.

### Neutral

- **Deep Space** (`oklch(9% 0.003 245)`, approx `#0f1012`): The application background and the photo grid canvas. Near-black with an imperceptible cool tint toward the primary hue — never a raw `#000000`.
- **Surface** (`oklch(14% 0.003 245)`, approx `#1c1e20`): Photo cards, panel backgrounds, and list containers. Distinguishable from the background without competing with content.
- **Lifted Slate** (`oklch(18% 0.003 245)`, approx `#252729`): Elevated surfaces — the nav rail background, dialog cards, toolbar fill. The tonal step that signals interactivity without a shadow.
- **Cold White** (`oklch(91% 0.005 245)`, approx `#e3e5e8`): Primary body text, labels, and icon fills. Slightly cool, slightly off-white. Never pure white.
- **Dim Ash** (`oklch(63% 0.005 245)`, approx `#979ba0`): Secondary text, placeholder text, inactive icon states, dividers.

### Error

- **Signal Red** (`oklch(58% 0.13 14)`, approx `#cc6070`): Error states and invalid tag indicators only. Never used for decoration.

### Named Rules

**The One Voice Rule.** The primary accent appears on ≤15% of any given screen. Its rarity is the point: when Observatory Slate appears, it means something is active or selected. Ambient saturation destroys that signal.

**The No Raw Black Rule.** Never use `#000` or `#fff`. Every neutral is tinted toward the primary hue (chroma 0.003–0.005). The effect is imperceptible but prevents the cheapness of a raw terminal palette.

**The User Chaos Rule.** Tags carry user-assigned colors. These are content, not system design. The system does not govern them — it accommodates them. Tag chips display user colors without constraint. The dark background absorbs most color combinations without visual conflict.

## 3. Typography

**Body Font:** Roboto (with `sans-serif` fallback)

Single-family throughout. No display serif, no mono for labels — this is a tool, not an editorial product. Hierarchy is established entirely through weight (400 vs. 500) and size contrast. Roboto at these weights on a dark background reads with the understated precision of instrument panel typography.

### Hierarchy

- **Headline** (500, 20px, line-height 1.3): Section headers, dialog titles, page-level identifiers. Appears rarely — one per major surface.
- **Title** (500, 16px, line-height 1.4): Subsection labels, toolbar component identifiers. The primary label weight for any named grouping.
- **Body** (400, 14px, line-height 1.5, tracking 0.015em): All descriptive text, list item labels, metadata fields. Cap line length at 65ch in flowing text contexts.
- **Label** (500, 12px, line-height 1.4, tracking 0.04em): Chips, tag badges, icon button labels, counter text. Minimum readable size in the system — do not go below 12px.

### Named Rules

**The Flat Scale Rule.** There is no display tier. This is a tool. The maximum headline size is 20px. If something needs to feel large, it does so through whitespace and position, not a 48px font. Resist the urge to add a "hero" size to any app screen.

## 4. Elevation

Flat by default. Depth is expressed through tonal layering — each surface tier is a distinct `oklch` lightness step — not through box shadows. The three tiers are: background (9%), surface (14%), lifted (18%). This progression is sufficient to distinguish the app shell, content panels, and elevated surfaces.

Shadows appear in one context only: floating menus (`v-menu`) and modal dialogs. Here a diffuse ambient shadow signals that a layer is above the main canvas.

### Shadow Vocabulary

- **Ambient dialog** (`0 8px 32px oklch(5% 0 0 / 0.6)`): Used on dialogs and floating menus. Diffuse, dark, not colored. Indicates modal elevation. Not used on cards, toolbars, or the nav rail.

### Named Rules

**The Flat-By-Default Rule.** Surfaces are flat at rest. Shadows appear only in modal contexts (dialogs, menus). A card that lifts on hover is a consumer UI pattern — prohibited here. Tonal steps, not shadow elevation, distinguish surfaces.

## 5. Components

### Navigation Rail

The structural spine of the app. Icon-only at rest (56px wide); expands to show labels on hover (256px). The expand-on-hover interaction is handled by Vuetify's rail drawer — no custom motion is needed. The bottom section holds Settings and Close Project, separated from the main nav by a divider.

- **Shape:** No border-right or outline. The background step (Lifted Slate, 18%) separates it from the canvas implicitly.
- **Inactive item:** Icon in Dim Ash (63%), no background.
- **Hover item:** Icon shifts to Cold White (91%), subtle background tint (`oklch(91% 0.005 245 / 0.08)`).
- **Active item:** Observatory Slate fill (primary), icon and label in Deep Space (background). Rounded (8px).

### Toolbar

Full-width bar across the top of the primary content pane. On the main tagger view, it uses primary color (Observatory Slate). On secondary panes (detail header), it uses Lifted Slate for tonal distinction.

- **Search field:** Outlined variant inside the toolbar. Compact density. Chips display active filter tokens in the field.
- **Icon buttons:** Flat, icon-only. No visible border. Color matches toolbar context.
- **Dividers:** Not used inside toolbars — spacing and grouping handle separation.

### Photo Grid

A virtual-scrolled grid of square photo tiles. The grid is the center of mass of the entire application.

- **Tile:** Square card with no border radius. The photo fills the tile entirely (cover). Filename fades in on hover via opacity transition (100ms ease-out). Status icons (rating stars, tag indicator, location marker) appear in a semi-transparent strip at the bottom of the image.
- **Selected state:** A checkmark icon overlay — no color border or outline around the card.
- **No hover lift.** Photos do not scale or gain shadow on hover. The name reveal is the only hover response.

### Buttons

Buttons appear in two functional contexts: dialogs (primary action + cancel) and toolbars (icon-only).

- **Primary button:** Observatory Slate background, Deep Space text. 4px radius. Uppercase label, 500 weight, 0.04em tracking. Hover shifts lightness to 70%.
- **Ghost button (cancel/secondary):** Transparent background, Cold White text. Same shape. Hover adds a 8% white tint background. Used in dialog action rows alongside a primary.
- **Icon button (toolbar):** No background at rest. Icon in Cold White. Compact density. No border radius styling needed — icon shape defines the hit area.

### Tag Chips

Tags appear in two contexts: the search combobox (as filter tokens) and the photo detail panel (as applied tags). User-assigned colors are respected without constraint.

- **Shape:** Fully rounded pill (9999px). 12px label, 500 weight, 0.04em tracking.
- **Default (no user color):** Lifted Slate background, Cold White text.
- **User-colored:** Background is the user's chosen color. Text is always Cold White regardless of background color (the dark surfaces around chips provide sufficient contrast context).
- **No hover interaction on chips.** They are display elements in the detail panel; interactive only in the search combobox.

### Inputs and Fields

- **Variant:** Outlined throughout (`variant="outlined"` in Vuetify). No filled or underline variants.
- **Default border:** Dim Ash (63%). Rounded 4px.
- **Focus border:** Observatory Slate (primary). No glow — border color shift only.
- **Placeholder text:** Dim Ash (63%).
- **Error border:** Signal Red.

### Dialogs

- **Background:** Lifted Slate (18%). Distinguishes from both the canvas and surface tiers.
- **Max width:** 80vw. Never full-screen for standard forms.
- **Structure:** Title (Headline, 20px, 500) → Content slot → Action row (right-aligned: Cancel ghost + Primary action).
- **Border radius:** 8px. The one place in the UI where a moderate radius is appropriate — it visually separates the dialog from the sharp-edged main surfaces.
- **Shadow:** Ambient dialog shadow only.

### Tag Graph (Signature Component)

The D3 force-directed graph on the Tags page. Nodes represent tags; links represent tag relationships.

- **Nodes:** Filled with Surface-Light (Vuetify `surface-light` CSS variable, equivalent to Lifted Slate). Stroke 1px same color. No border radius — circular.
- **Links:** 1px stroke in Lifted Slate. Relationship type determines stroke-dasharray or weight.
- **Labels:** 10px Roboto, fill Cold White, stroke Deep Space with `paint-order: stroke` for readability against varied node colors.
- **Background:** Deep Space canvas. The graph floats on the application background directly — no card container.

## 6. Do's and Don'ts

### Do:

- **Do** use Observatory Slate only for active/selected states, primary actions, and focused input borders. Everywhere else, rely on the neutral tonal scale.
- **Do** let photos fill their tiles completely. Aspect-ratio 1/1, cover fit. The image is the content — don't frame it.
- **Do** use font-weight (400 vs. 500) as the primary hierarchy tool within a size step. A 14px/500 label reads as more prominent than a 14px/400 body without needing a size change.
- **Do** let the nav rail communicate active state via the primary color chip. It is the only place where a filled background on a navigation item is appropriate.
- **Do** use the toolbar to compress controls. Everything a user needs for the current context lives in the toolbar — no floating action buttons, no sidebars for primary actions.
- **Do** use tonal surface steps (background → surface → lifted) to separate layers. This is elevation without shadows.
- **Do** maintain WCAG AA contrast (4.5:1 body, 3:1 large text) across all text/background combinations. Cold White on Deep Space is the baseline.

### Don't:

- **Don't** use green (`#4caf50`) as the primary accent. The stock Material green is a placeholder we replaced. Any PR that reintroduces it reverts to generic Material Design with no personality.
- **Don't** use card grids where every card is the same size with icon + heading + text. The photo grid is the only card grid in this app, and it is photo-first. Admin-style card dashboards are prohibited.
- **Don't** apply `border-left` or `border-right` greater than 1px as a colored accent stripe on any element. Never on list items, callouts, or panels. Rewrite with background tints or no decoration.
- **Don't** use gradient text or `background-clip: text` with a gradient fill. All text is a single solid color.
- **Don't** use floating action buttons (FABs). All primary actions live in the toolbar or in dialog action rows.
- **Don't** design for infinite scroll or content discovery feeds. This is an archive, not a feed. Navigation is explicit and user-directed.
- **Don't** use pastel accents or soft, rounded consumer-app aesthetics (Google Photos / iCloud style). The target user knows what they're doing — the UI does not need to be approachable.
- **Don't** add modal dialogs as the first response to any user action. Exhaust inline and progressive options first. Dialogs are for irreversible multi-field operations only.
- **Don't** use decorative glassmorphism (blurred, frosted surfaces). If it appears, it's ornament. Remove it.
- **Don't** let the UI compete with the photo. If a photo is in view in the detail pane, the grid compresses to a column and chrome recedes. Any design that lets toolbar color, card decoration, or accent saturation compete visually with the image is wrong.
