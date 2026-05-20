# Demo Video — Shot List

*60–75 seconds. No music. No voiceover unless you record one
intentionally. Terminal + browser only. The point is to make the
"AI emits IR → IR compiles to a real page" loop legible in under a
minute. Optimized to embed on the blog, ProductHunt page, and the
top of the README.*

---

## Format

- **Length target:** 65 seconds.
- **Aspect ratio:** 16:9, 1080p. Re-crop a 9:16 vertical for X /
  ProductHunt mobile.
- **Terminal:** standard dark theme, 14pt monospace, no shell prompt
  decoration beyond `$`. Hide unnecessary clutter (timestamps, git
  branch in PS1).
- **Browser:** clean profile, no extensions visible, 1280×720 window
  centered.
- **Cursor:** record real keystrokes; do not synth-type. The
  natural typing rhythm is the credibility cue.
- **Cuts:** straight cuts, no transitions. Aim for ~7 scenes.

## Scene plan

### 0:00–0:04 — Title card (4s)

Static frame on a clean black background:

```
voce-ir
AI-native UI without human-readable code
```

(Project logo if one is finalized; otherwise plain type.)

### 0:04–0:14 — Conversational generation (10s)

Terminal. Type:

```
$ voce-chat
```

Wait for the prompt. Type a natural-language brief, slowly:

```
> Build a contact page with a form (email + message),
  a clear "Send" button, and proper accessibility.
```

The assistant should respond with **one discovery question** (the
five-phase workflow's quality gate). Screenshot evidence: a real
voce-chat session producing a single clarifying question, not a
full IR dump.

### 0:14–0:26 — IR appears (12s)

The assistant runs `voce_generate_propose` and prints the JSON IR.
Cut to a split or zoom showing the typed IR shape — `value_type`
fields, `semantic_node_id` references, `validations` array on the
email field.

Annotation overlay (one short line):
*"the AI's output is typed IR, not source code"*

### 0:26–0:36 — Validate + fix (10s)

Switch terminals (or new pane). Run:

```
$ voce validate contact.voce.json
✓ 9 passes, 52 rules, 0 errors
```

Then deliberately break it — open the IR in an editor, remove the
form's `semantic_node_id`. Re-run validate:

```
$ voce validate contact.voce.json
✗ A11Y001  FormNode must have a semantic_node_id for accessibility
  hint: Add a SemanticNode with role:"form"...
```

Then:

```
$ voce fix contact.voce.json --until-clean --apply
voce fix --until-clean (contract v1.0.0)
  Iterations: 2   converged: true
  Plan: 1 step
    1. [safe] STR002 at /root/children/0
       → Add node_id "root-children-0" to /root/children/0
```

Annotation overlay: *"accessibility is a compile error"*

### 0:36–0:50 — Compile + preview (14s)

```
$ voce compile contact.voce.json
✓ dist/contact.html (7.8 KB, zero runtime deps)

$ voce preview contact.voce.json
```

Cut to the browser. The page is plain, fast, and visibly accessible:
focus rings work when you Tab through. **Capture the page rendering
in under a second** — that's the proof of "zero runtime deps."

Open dev tools network tab to show: 1 HTML file, no JS, no
framework runtime.

### 0:50–0:60 — Agent contract glimpse (10s)

Back to terminal:

```
$ voce skills
Voce capability manifest
  contract: v1.0.0
  voce:     v0.1.0

Validation passes (9): structural, references, state-machine,
  accessibility, security, seo, forms, i18n, motion
Diagnostic codes: 52 total (17 fixable)
Node types: 27
Compile targets (7):
  - dom              Stable
  - hybrid           Stable
  - email            Stable
  - webgpu           Beta
  - ...
```

Annotation overlay: *"the agent contract is the only interface — so
we made it complete"*

### 0:60–0:65 — End card (5s)

Three-line frame:

```
voce-ir.xyz
github.com/marc-pelland/voce-ir
npm install -g @voce-ir/cli-chat
```

## What NOT to film

- **Don't film waiting on a network round-trip** — the LLM call
  latency is the most boring part. Cut tight or pre-stage so the
  assistant's first response is already underway.
- **Don't film the Anthropic API key being typed.** Pre-export
  `ANTHROPIC_API_KEY` in the recording shell so it's invisible.
- **Don't film a happy-path that ignores validation.** The
  show-the-validator-stops-the-build moment is the differentiator;
  cutting it makes the video generic.
- **Don't speed-ramp the typing.** Natural rhythm is the credibility
  signal; 2× type-speed reads as fake even if you'd never name it.

## Render checklist

- 1080p H.264, single audio track (silence is fine).
- Export with chapters / cuepoints at scene boundaries so the blog
  post can deep-link.
- A 9:16 1080×1920 re-crop for X and PH mobile (focus on terminal +
  browser zone; the title and end cards reflow cleanly).
- Captions burned in for the annotation overlays (a11y for the
  a11y demo — closes the loop).
