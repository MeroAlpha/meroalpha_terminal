---
version: alpha
name: MeroAlpha Data Platform
description: Dark-only developer-first NEPSE market-data platform with restrained
  emerald accents, stable API previews, and clear no-execution boundaries.
colors:
  background: '#09090B'
  surface: '#111113'
  surfaceContainer: '#1C1C1F'
  border: '#27272A'
  text: '#FAFAFA'
  textMuted: '#A1A1AA'
  emeraldAccent: '#34D399'
  emeraldSoft: rgba(52, 211, 153, 0.10)
  surface-dim: '#141313'
  surface-bright: '#3a3939'
  surface-container-lowest: '#0e0e0e'
  surface-container-low: '#1c1b1b'
  surface-container: '#1C1C1F'
  surface-container-high: '#2a2a2a'
  surface-container-highest: '#353434'
  on-surface: '#e5e2e1'
  on-surface-variant: '#c4c7c8'
  inverse-surface: '#e5e2e1'
  inverse-on-surface: '#313030'
  outline: '#8e9192'
  outline-variant: '#444748'
  surface-tint: '#c6c6c7'
  primary: '#ffffff'
  on-primary: '#2f3131'
  primary-container: '#e2e2e2'
  on-primary-container: '#636565'
  inverse-primary: '#5d5f5f'
  secondary: '#45dfa4'
  on-secondary: '#003825'
  secondary-container: '#00bd85'
  on-secondary-container: '#00452e'
  tertiary: '#ffffff'
  on-tertiary: '#32302d'
  tertiary-container: '#e7e1dd'
  on-tertiary-container: '#676460'
  error: '#ffb4ab'
  on-error: '#690005'
  error-container: '#93000a'
  on-error-container: '#ffdad6'
  primary-fixed: '#e2e2e2'
  primary-fixed-dim: '#c6c6c7'
  on-primary-fixed: '#1a1c1c'
  on-primary-fixed-variant: '#454747'
  secondary-fixed: '#68fcbf'
  secondary-fixed-dim: '#45dfa4'
  on-secondary-fixed: '#002114'
  on-secondary-fixed-variant: '#005137'
  tertiary-fixed: '#e7e1dd'
  tertiary-fixed-dim: '#cbc6c1'
  on-tertiary-fixed: '#1d1b19'
  on-tertiary-fixed-variant: '#494643'
  on-background: '#e5e2e1'
  surface-variant: '#353434'
  text-muted: '#A1A1AA'
  emerald-soft: rgba(52, 211, 153, 0.10)
typography:
  sans: Geist Variable
  mono: JetBrains Mono
  headline-lg:
    fontFamily: Geist
    fontSize: 30px
    fontWeight: '600'
    lineHeight: 36px
    letterSpacing: -0.02em
  headline-md:
    fontFamily: Geist
    fontSize: 24px
    fontWeight: '600'
    lineHeight: 32px
    letterSpacing: -0.01em
  headline-sm:
    fontFamily: Geist
    fontSize: 18px
    fontWeight: '500'
    lineHeight: 28px
  body-lg:
    fontFamily: Geist
    fontSize: 16px
    fontWeight: '400'
    lineHeight: 24px
  body-md:
    fontFamily: Geist
    fontSize: 14px
    fontWeight: '400'
    lineHeight: 20px
  data-mono:
    fontFamily: JetBrains Mono
    fontSize: 13px
    fontWeight: '400'
    lineHeight: 18px
  label-caps:
    fontFamily: JetBrains Mono
    fontSize: 11px
    fontWeight: '600'
    lineHeight: 16px
    letterSpacing: 0.05em
  code-sm:
    fontFamily: JetBrains Mono
    fontSize: 12px
    fontWeight: '400'
    lineHeight: 16px
rounded:
  control: 0.5rem
  panel: 0.75rem
  pill: 9999px
  sm: 0.25rem
  DEFAULT: 0.5rem
  md: 0.75rem
  lg: 1rem
  xl: 1.5rem
  full: 9999px
spacing:
  margin-desktop: 24px
  margin-mobile: 16px
  gutter: 16px
  stack-compact: 8px
  stack-relaxed: 24px
---

## Overview
MeroAlpha Data Platform is a developer-first market-data surface for NEPSE datasets. The product should feel precise, stable, and operational: developers are inspecting API envelopes, requesting access, and importing historic CSVs, not placing trades or receiving portfolio advice.
Keep the interface dark-only. Use quiet panels, crisp borders, compact data typography, and restrained emerald accents for market-data signal. Avoid decorative effects that add runtime complexity without explaining the data product.
## Visual system
- **Background:** near-black app canvas with subtle section separation.
- **Surfaces:** dark zinc panels with one border layer; avoid stacked glassmorphism.
- **Accent:** emerald only for market-signal dots, section accents, active states, and restrained callouts.
- **Primary CTAs:** neutral/white hierarchy unless intentionally changing conversion weight.
- **Typography:** Use sans for headings/labels and mono for symbols, endpoints, dates, API keys, and numeric market data.
- **Motion:** keep simple CSS transitions.