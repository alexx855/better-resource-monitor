---
version: alpha
name: Better Resource Monitor
description: A dark, monospace, utility-first marketing design system for a lightweight macOS menu bar monitor.
colors:
  primary: "#a93a12"
  primary-hover: "#822d0e"
  focus: "#ff8a4c"
  background: "#18120f"
  surface: "#211914"
  surface-alt: "#2a211c"
  border: "#4a382e"
  text: "#fff8f2"
  text-muted: "#dcc9bc"
  on-primary: "#ffffff"
  selection-text: "#1a0f09"
typography:
  display-xl:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 4.9rem
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: 0em
  display-xl-mobile:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 2.4rem
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: 0em
  headline-lg:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 2.4rem
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: 0em
  headline-md:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 1.65rem
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: 0em
  title-md:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 1.25rem
    fontWeight: 600
    lineHeight: 1.6
    letterSpacing: 0em
  body-lg:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 1.125rem
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: 0em
  body-md:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 1rem
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: 0em
  body-sm:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 0.875rem
    fontWeight: 400
    lineHeight: 1.6
    letterSpacing: 0em
  label-caps:
    fontFamily: ui-monospace, Cascadia Code, Source Code Pro, Menlo, monospace
    fontSize: 0.75rem
    fontWeight: 600
    lineHeight: 1.6
    letterSpacing: 0.05em
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
  border-width: 1px
  focus-width: 4px
rounded:
  none: 0px
  sm: 0px
  md: 0px
  lg: 0px
  full: 0px
shadows:
  none: none
  focus-ring: 0 0 0 4px #ff8a4c
elevation:
  base:
    backgroundColor: "{colors.background}"
    shadow: "{shadows.none}"
  surface:
    backgroundColor: "{colors.surface}"
    borderColor: "{colors.border}"
    borderWidth: "{spacing.border-width}"
    shadow: "{shadows.none}"
  surface-hover:
    borderColor: "{colors.primary}"
    shadow: "{shadows.none}"
motion:
  ease-out: cubic-bezier(0.23, 1, 0.32, 1)
  ease-in-out: cubic-bezier(0.77, 0, 0.175, 1)
  fast: 150ms
  normal: 200ms
  active-scale: 0.97
components:
  page:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text}"
    typography: "{typography.body-md}"
  container:
    width: "{spacing.container-max}"
    padding: "{spacing.6}"
  link:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    padding: 2px 4px
  link-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.on-primary}"
  button-like-link:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-sm}"
    rounded: "{rounded.none}"
    height: "{spacing.touch-target}"
    padding: 2px 4px
  card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.none}"
    padding: "{spacing.5}"
  card-hover:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
  table:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.none}"
  table-header:
    backgroundColor: "{colors.surface-alt}"
    textColor: "{colors.text}"
    typography: "{typography.label-caps}"
    padding: 12px 24px
  table-cell:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    typography: "{typography.body-md}"
    padding: 12px 24px
  code-block:
    backgroundColor: "{colors.surface-alt}"
    textColor: "{colors.text}"
    rounded: "{rounded.none}"
    padding: "{spacing.5}"
  blockquote:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text-muted}"
    rounded: "{rounded.none}"
    padding: 12px 24px
  footer-link:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-caps}"
    rounded: "{rounded.none}"
    height: "{spacing.touch-target}"
    padding: 2px 4px
  focus-indicator:
    backgroundColor: "{colors.focus}"
    textColor: "{colors.selection-text}"
    rounded: "{rounded.none}"
    size: "{spacing.focus-width}"
  divider:
    backgroundColor: "{colors.border}"
    size: "{spacing.border-width}"
---

# Better Resource Monitor Design System

## Overview

Better Resource Monitor uses a compact, dark, technical visual system that feels closer to a native utility readout than a conventional SaaS landing page. The design should communicate low overhead, privacy, and practical system insight. It is intentionally restrained: one warm accent, one monospace family, hard edges, clear tables, and direct copy.

The interface should feel trustworthy, efficient, and quiet. Avoid ornamental polish that would make the product feel heavier than the app it describes. The visual identity is strongest when it looks like a refined diagnostic panel: readable, dark, warm, minimal, and precise.

## Colors

The palette is a warm dark mode system with copper-orange interactions.

- **Background (#18120f):** The primary canvas, a deep roasted brown-black that avoids neutral gray coldness.
- **Surface (#211914):** Used for tables, FAQ cards, blockquotes, and content containers that need separation from the page.
- **Surface Alt (#2a211c):** A raised tonal layer for table headers, code blocks, and inline code.
- **Border (#4a382e):** A muted brown divider that defines structure without heavy contrast.
- **Primary (#a93a12):** The only strong interaction color. Use it for links, calls to action, blockquote bars, and hover borders.
- **Primary Hover (#822d0e):** A darker copper-brown state for hovered links.
- **Focus (#ff8a4c):** A bright orange keyboard focus ring and text selection color.
- **Text (#fff8f2):** Warm near-white for headings and strong content.
- **Text Muted (#dcc9bc):** Warm beige for body copy and secondary text.

The background is not flat. It combines the base color with subtle warm radial glows and a faint diagonal 18px grid. That texture should stay quiet and low contrast; it should never compete with the tray image, headings, tables, or app badges.

## Typography

Typography is entirely monospace. This is central to the product identity because it mirrors system metrics, terminal output, and lightweight diagnostics. Use the platform monospace stack with `Cascadia Code`, `Source Code Pro`, and `Menlo` fallbacks.

Headings use the same family as body text, differentiated only by size, weight, spacing, and position. They are semibold, warm near-white, and balanced across lines. Body text is regular weight with generous 1.6 line height for readability. Labels, table headers, and footer navigation use uppercase treatment with 0.05em tracking.

Large homepage headings may scale fluidly up to the display size, but content pages should remain more compact. Do not introduce a separate sans-serif display face; the single-family system is part of the brand.

## Layout

The layout is a single-column document system with a maximum content width of 900px. Pages are centered with responsive padding that ranges from 16px to 32px, and page rhythm follows a 4px-based spacing scale with common steps at 8px, 12px, 16px, 24px, 32px, and 64px.

Content should be direct and scannable. The home page is essentially a styled document with a prominent product tray image, centered badges, a compact link row, then practical sections and comparison tables. Secondary pages use the same constrained column, starting with a small back link and then content.

Tables are important first-class components. They should feel dense enough for comparison, with horizontal scrolling on small screens and stable 1px grid lines. Do not replace comparison data with card grids unless the table becomes unusable.

## Elevation & Depth

Depth is intentionally flat. The system does not use drop shadows, glass, blur panels, or soft card elevation. Hierarchy comes from warm surface colors, 1px borders, copper interaction states, uppercase labels, and spacing.

The only visual lift is motion on interactive links: hover translates them upward by 1px, image links brighten slightly, and active links scale to 0.97. Keyboard focus is intentionally loud: a 4px solid orange outline with a 2px offset.

## Shapes

The shape language is square and utilitarian. Corners are 0px across links, cards, code blocks, tables, blockquotes, and footer controls. This hard-edged geometry reinforces the diagnostic, resource-monitoring identity.

Avoid adding rounded cards, pill buttons, floating panels, or soft containers. If a future component needs stronger affordance, use border weight, fill, spacing, or typographic hierarchy before changing the corner radius.

## Components

**Links and calls to action** are solid copper rectangles with white text. They use very tight padding, including a slight negative horizontal margin in prose so inline links feel like highlighted terminal text rather than traditional underlined links. Hover darkens the fill and shifts the link up 1px.

**Image links** are unframed. App Store, macOS, and product images should keep their native artwork and should only receive subtle hover brightness or contrast treatment.

**FAQ cards** are square surface blocks with 1px brown borders and 24px padding. On hover, only the border changes to copper. The card content remains quiet and readable.

**Tables** use surface backgrounds, surface-alt headers, uppercase header labels, 12px by 24px cell padding, and 1px brown dividers. Table body row headings return to normal case and normal letter spacing.

**Code** uses the same monospace family as all other text. Inline code sits on the alternate surface with a 1px border and compact padding. Code blocks use 24px padding and horizontal scrolling.

**Blockquotes** are surface panels with a thick 22px copper left bar. They are used as warning, caveat, or note treatments rather than decorative quotes.

**Footer navigation** is a row of uppercase copper links with 44px minimum target height. On narrow screens it stacks vertically with 12px gaps.

## Do's and Don'ts

- Do keep the site dark, warm, monospace, and technically direct.
- Do use copper-orange as the single dominant action color.
- Do preserve square corners and 1px borders for structural UI.
- Do keep comparison tables readable and horizontally scrollable on small screens.
- Do use bright orange focus outlines for keyboard accessibility.
- Don't introduce rounded cards, pastel gradients, glass effects, or soft shadows.
- Don't add extra typefaces or decorative display typography.
- Don't use generic blue links or underlines; links should read as warm highlighted blocks.
- Don't make marketing sections feel like floating cards; the site should remain document-like and utility-focused.
