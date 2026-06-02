---
name: Fidelity Modern
colors:
  surface: '#fff8f7'
  surface-dim: '#f3d3d1'
  surface-bright: '#fff8f7'
  surface-container-lowest: '#ffffff'
  surface-container-low: '#fff0ef'
  surface-container: '#ffe9e8'
  surface-container-high: '#ffe1e0'
  surface-container-highest: '#fcdbd9'
  on-surface: '#281717'
  on-surface-variant: '#5d3f3e'
  inverse-surface: '#3f2b2b'
  inverse-on-surface: '#ffedeb'
  outline: '#926e6d'
  outline-variant: '#e7bdbb'
  surface-tint: '#bf0029'
  primary: '#bc0028'
  on-primary: '#ffffff'
  primary-container: '#e61938'
  on-primary-container: '#fffdff'
  inverse-primary: '#ffb3b1'
  secondary: '#ac3138'
  on-secondary: '#ffffff'
  secondary-container: '#fc6d6f'
  on-secondary-container: '#6d0012'
  tertiary: '#006770'
  on-tertiary: '#ffffff'
  tertiary-container: '#00828d'
  on-tertiary-container: '#fbffff'
  error: '#ba1a1a'
  on-error: '#ffffff'
  error-container: '#ffdad6'
  on-error-container: '#93000a'
  primary-fixed: '#ffdad8'
  primary-fixed-dim: '#ffb3b1'
  on-primary-fixed: '#410007'
  on-primary-fixed-variant: '#92001d'
  secondary-fixed: '#ffdad8'
  secondary-fixed-dim: '#ffb3b1'
  on-secondary-fixed: '#410007'
  on-secondary-fixed-variant: '#8b1823'
  tertiary-fixed: '#92f1fd'
  tertiary-fixed-dim: '#75d5e1'
  on-tertiary-fixed: '#001f23'
  on-tertiary-fixed-variant: '#004f56'
  background: '#fff8f7'
  on-background: '#281717'
  surface-variant: '#fcdbd9'
typography:
  headline-lg:
    fontFamily: Inter
    fontSize: 32px
    fontWeight: '700'
    lineHeight: 40px
  headline-md:
    fontFamily: Inter
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
  body-lg:
    fontFamily: Inter
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Inter
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  label-md:
    fontFamily: Inter
    fontSize: 12px
    fontWeight: '500'
    lineHeight: 16px
    letterSpacing: 0.5px
rounded:
  sm: 0.5rem
  DEFAULT: 1rem
  md: 1.5rem
  lg: 2rem
  xl: 3rem
  full: 9999px
spacing:
  base: 8px
  xs: 4px
  sm: 8px
  md: 16px
  lg: 24px
  xl: 32px
  gutter: 16px
  margin: 24px
---

# Fidelity Modern Design System

## Brand & Style
Fidelity Modern is a professional, high-energy, and reliable design system. It balances a striking, high-fidelity red primary palette with a clean, modern aesthetic that feels both corporate and approachable. The style is **Corporate / Modern**, prioritizing clarity, precision, and a sense of forward-moving momentum. It evokes trust through balanced layouts while maintaining a distinct personality through its vibrant accent colors and approachable, pill-shaped geometry.

## Colors
The color palette is anchored by a vibrant Primary Red, designed to draw attention and signal action. The secondary and tertiary tones provide depth and functional variety.

*   **Primary (#e91c3a):** Used for key brand moments, primary calls-to-action, and critical states.
*   **Secondary (#cd4a4e):** A more muted red for supplementary elements and secondary actions.
*   **Tertiary (#00828d):** A deep teal used for balance, information callouts, and distinct interactive elements.
*   **Neutral (#8b7170):** A warm, grounded grey used for surfaces, borders, and text to ensure readability and softness against the high-energy primary colors.

## Typography
The system uses **Inter** for all typographic layers. Inter provides exceptional legibility on screens, a neutral yet modern character, and a wide range of weights that allow for clear information hierarchy. 

*   **Headlines:** Assertive and bold, utilizing Inter's heavier weights (600-700).
*   **Body:** Clean and highly readable, optimized for long-form content and UI labels.
*   **Labels:** Small, medium-weight text with slight letter spacing for maximum clarity in tight spaces.

## Layout & Spacing
The system utilizes a **Fluid Grid** model based on an 8px spacing rhythm. Content is organized to scale seamlessly across devices, using 16px gutters and 24px margins as standard container padding. The 8px base unit ensures consistent vertical and horizontal alignment, creating a predictable and balanced visual flow.

## Elevation & Depth
Hierarchy is established primarily through **Tonal Layers** and soft, ambient shadows. Surfaces use subtle shifts in the neutral palette to indicate stacking, while interactive elements like cards and floating buttons employ low-opacity, diffused shadows to lift them from the background without creating visual clutter.

## Shapes
The design language features a **Pill-shaped** geometry. This high level of roundedness (Level 3) creates an extremely approachable, friendly, and modern feel. 

*   **Standard Elements:** UI components like buttons and inputs use a 1rem (16px) radius.
*   **Large Containers:** Cards, dialogs, and modals use 2rem (32px) or 3rem (48px) radii to maintain the soft, organic aesthetic.

## Components
*   **Buttons:** Fully pill-shaped with significant horizontal padding. Use the Primary color for high-emphasis actions.
*   **Input Fields:** Soft, rounded corners (1rem) with a 1px neutral border that thickens or changes color on focus.
*   **Cards:** Large corner radii (2rem+) with subtle ambient shadows or light neutral backgrounds.
*   **Chips & Labels:** Fully rounded (pill) containers used for tagging, filtering, and status indicators.
*   **Checkboxes & Radios:** High-contrast selections using the primary or tertiary colors to ensure clear user feedback.