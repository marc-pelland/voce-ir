# Manual Accessibility Testing

The validator proves what is statically decidable. It cannot prove the
*lived experience*. These checks are human-owned and required before a
release that touches UI output. This is by design — automating them
would give false confidence.

## When to run

- Before any release that changes a compiler's HTML/SwiftUI/Compose
  emission.
- When adding or changing an A11Y rule (sanity-check it against a real
  screen reader, not just unit tests).

## Screen readers

| Platform | Screen reader | Browser |
| --- | --- | --- |
| macOS | VoiceOver (⌘F5) | Safari |
| Windows | NVDA (free) | Firefox / Chrome |
| Windows | JAWS (if available) | Chrome |
| iOS | VoiceOver | Safari (for the SwiftUI target) |
| Android | TalkBack | (for the Compose target) |

## Checklist (per representative page)

1. **Heading navigation** — jump by heading (VO: `Ctrl+Opt+Cmd+H`).
   Order and levels match the visual structure; no skips.
2. **Landmark navigation** — rotor/landmarks lists nav, main,
   contentinfo as expected.
3. **Tab order** — Tab through every interactive element. Order is
   logical; focus is always visible; no keyboard trap.
4. **Links/buttons** — each announces a meaningful name (not "link",
   "button", or a raw URL). Icon-only controls have a real name.
5. **Forms** — every field announces its label; required state and
   validation errors are announced (not just shown).
6. **Status messages** — dynamic updates (fetch results, form errors)
   are announced without moving focus (`LiveRegion`).
7. **Images** — meaningful images announce their purpose; decorative
   images are silent.
8. **Reduced motion** — with OS "reduce motion" on, animation is
   suppressed.
9. **Zoom / reflow** — 200% zoom and 320px width: no loss of content
   or function (validator does not check this).

## Recording results

Per release, capture pass/fail + notes per checklist item per tested
SR/browser pair. Keep it alongside the release notes. The continuously
machine-checked corpus evidence is in `EVIDENCE.md`; this human pass is
the part that file explicitly does **not** claim to cover.
