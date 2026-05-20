---
version: alpha
name: Better Resource Monitor
description: A dark, monospace, utility-first marketing design system for a lightweight macOS menu bar monitor.
colors:
  brand: "#D14715"
  background: "#18120f"
  surface: "#211914"
  surface-alt: "#2a211c"
  border: "#4a382e"
  text: "#fff8f2"
  text-muted: "#dcc9bc"
  on-brand: "#ffffff"
typography:
  display:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: clamp(2.6rem, 9vw, 4.9rem)
    fontWeight: 600
    lineHeight: 1.2
  heading:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: clamp(1.85rem, 5vw, 2.4rem)
    fontWeight: 600
    lineHeight: 1.2
  body:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 1rem
    fontWeight: 400
    lineHeight: 1.6
spacing:
  1: 4px
  2: 8px
  3: 12px
  4: 16px
  5: 24px
  6: 32px
  10: 64px
  container-max: 900px
  touch-target: 44px
shapes:
  radius: 0px
motion:
  fast: 150ms
  normal: 200ms
---

# Better Resource Monitor Design System

## Overview

Better Resource Monitor should feel like a refined macOS utility readout: dark, compact, monospace, direct, and low overhead. The site is a styled technical document, not a soft SaaS landing page.

Keep the identity simple:

- One brand color: `#D14715`, verified from the app tray artwork.
- One type family: the platform monospace stack.
- One layout model: a centered document column.
- One shape language: square edges and 1px structure.

## Color

Use `#D14715` as the only brand color. It should cover links, calls to action, focus rings, selection, code/block quote accents, hover borders, and small emphasis marks.

Do not add separate primary, hover, focus, warning, accent, or secondary brand colors. If a state needs variation, derive it from the brand color with opacity or a darker mix in CSS.

Core neutrals:

- `#18120f` for the page background.
- `#211914` for surfaces.
- `#2a211c` for raised/code/table-header surfaces.
- `#4a382e` for borders and dividers.
- `#fff8f2` for headings and strong text.
- `#dcc9bc` for body and muted text.
- `#ffffff` for text on the brand color.

The background may use subtle texture or warm glows, but all warm color should come from the brand color.

## Typography

Typography is entirely monospace. Use `ui-monospace`, `Cascadia Code`, `Source Code Pro`, `Menlo`, and `monospace` fallbacks.

Use weight, size, spacing, and position for hierarchy. Do not introduce a separate display face. Labels and table headers can use uppercase with light tracking.

## Layout

Use a single centered column with a max width of `900px`. Page padding should scale from `16px` to `32px`. Rhythm follows a 4px grid with common steps at `8px`, `12px`, `16px`, `24px`, `32px`, and `64px`.

The home page should stay document-like: tray image, badges, compact links, practical sections, FAQ content, and comparison tables. Secondary pages use the same column and start with a compact back link.

## Components

Links and calls to action are solid brand rectangles with white text and tight padding. Hover may darken the brand color slightly and move up by 1px. Active states may scale to `0.97`.

Cards and FAQ items are square surface blocks with 1px borders. Hover changes only the border to the brand color.

Tables use surface backgrounds, alternate header backgrounds, uppercase header labels, `12px 24px` cell padding, and 1px dividers. Keep comparison tables horizontally scrollable on small screens.

Code blocks use the alternate surface, monospace text, and a brand-colored left border. Inline code uses the same family with a subtle border.

Blockquotes are surface panels with a thick brand left bar. Use them for warnings, caveats, or notes rather than decorative quotes.

Footer navigation is a row of uppercase brand links with a `44px` minimum target height. On narrow screens it stacks vertically.

## Rules

- Do keep the site dark, warm, monospace, and direct.
- Do use `#D14715` as the only brand color.
- Do preserve square corners and 1px structure.
- Do keep tables readable and horizontally scrollable.
- Don't add rounded cards, pastel gradients, glass effects, or soft shadows.
- Don't add more typefaces.
- Don't add separate hover, focus, warning, or accent colors.
- Don't make sections feel like floating SaaS cards.
