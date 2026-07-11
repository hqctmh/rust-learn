# Turn Store Design QA

- Source visual truth: `docs/superpowers/specs/assets/2026-07-10-turn-store-ui-option-2.png`
- Primary implementation capture: `docs/superpowers/design-qa/turn-store-desktop-active.jpg`
- Responsive captures: `docs/superpowers/design-qa/turn-store-tablet.jpg`, `docs/superpowers/design-qa/turn-store-mobile.jpg`, `docs/superpowers/design-qa/turn-store-mobile-active.jpg`
- Full-view comparison: `docs/superpowers/design-qa/turn-store-full-comparison.jpg`
- Focused comparisons: `docs/superpowers/design-qa/turn-store-topbar-comparison.jpg`, `docs/superpowers/design-qa/turn-store-composer-comparison.jpg`
- Desktop viewport/state: 1440×1024, slow streaming conversation with user message, partial Agent response, composer, and visible event inspector.
- Responsive viewports/states: 959×900 empty state; 639×844 empty and completed conversation states.

## Findings

No actionable P0, P1, or P2 findings remain.

- [P3] Proprietary typefaces use system substitutes
  - Location: whole page.
  - Evidence: the source calls for Figma Sans/Figma Mono; the implementation uses Inter/system-ui and SF Mono/Menlo fallbacks.
  - Impact: small differences in glyph width and optical weight remain, but hierarchy and wrapping stay stable.
  - Follow-up: use licensed Figma fonts only if the project later receives local font assets.

- [P3] Decorative source marks are intentionally omitted
  - Location: source empty-state star and Agent avatar.
  - Evidence: the selected mock includes generated decorative marks; the production page uses text hierarchy and pastel surfaces without CSS-drawn or placeholder assets.
  - Impact: slightly less decorative personality, with no loss of task clarity.
  - Follow-up: add them only when a real icon library or approved image asset is available.

- [P3] Runtime content is denser than the concept mock
  - Location: Agent response and event inspector.
  - Evidence: the source uses a short example response; the real mock agent streams the full `DESIGN-figma.md` and hundreds of events.
  - Impact: the production state is denser, but independent scrolling keeps the composer and inspector usable.
  - Follow-up: consider event grouping or filtering as a separate product change.

## Required Fidelity Surfaces

- Fonts and typography: display hierarchy, tight heading tracking, mono taxonomy labels, body line height, wrapping, and truncation were checked. System substitutes are acceptable under the no-network-font constraint.
- Spacing and layout rhythm: the desktop main/inspector split, topbar, composer height, thin dividers, responsive gutters, and independent scrolling match the selected direction. The 959px and 639px captures have no page-level or horizontal overflow.
- Colors and visual tokens: white/black chrome, `#c8e6cd` inspector, `#c5b0f4` user message, `#f4ecd6` Agent message, `#e6e6e6` hairlines, and success green are present. No gradients or shadows remain.
- Image quality and asset fidelity: the interface does not require raster imagery. Decorative source marks were omitted rather than replaced with CSS art, emoji, text glyphs, inline SVG, or placeholders.
- Copy and content: product labels, Chinese controls, Conversation status, output mode, run events, and empty-state copy are coherent and preserve the existing workflow.

## Interaction and Accessibility Checks

- Browser-rendered checks covered: fast/slow selection, message submission, streaming disabled states, event inspector appearance, inspector collapse/expand, new-conversation reset, Enter/Shift+Enter semantics from existing handlers, and responsive layout.
- `aria-live`, textarea label, semantic `aside`, list semantics, `aria-expanded`, visible focus rules, 44px minimum button height, and disabled states are present.
- Browser console warnings/errors: none in the desktop run used for comparison.

## Comparison History

### Iteration 1

- [P2] The transparent topbar produced a black capture artifact in the active-state screenshot.
  - Fix: set `.topbar { background: var(--canvas); }` and add a regression contract test.
- [P2] At 959×900, the composer extended below the viewport.
  - Fix: keep `.shell` at `height: 100vh` for the tablet breakpoint; post-fix geometry reports composer bottom 868px in a 900px viewport.
- [P2] The empty message area showed an unnecessary inner scrollbar.
  - Fix: use `.empty-state { height: 100%; min-height: 0; }`; post-fix message scroll height equals client height at 536px.

### Iteration 2

- Post-fix desktop, tablet, mobile, and mobile-active captures were reviewed.
- Desktop source/implementation proportions, topbar, pastel surfaces, message alignment, inspector width, composer, and controls are materially aligned.
- Tablet geometry: page 959×900, composer bottom 868px, no page overflow, no empty-state inner overflow.
- Mobile geometry: page 639×844, composer bottom 828px, no horizontal/page overflow; expanded inspector occupies its own 320px row and collapsed inspector reduces to 71.5px.
- Earlier P2 findings are resolved; remaining differences are classified as P3 or intentional runtime/product constraints.

## Implementation Checklist

- [x] Preserve SSE API and conversation behavior.
- [x] Implement semantic desktop split layout.
- [x] Render structured event timeline.
- [x] Implement tablet/mobile responsive layouts.
- [x] Verify primary controls and inspector states.
- [x] Run full visual and automated regression checks.

## Follow-up Polish

- Optional licensed fonts or approved decorative assets can narrow the remaining P3 fidelity gap.
- Event grouping/filtering can be explored separately if real-world event volume becomes distracting.

final result: passed
