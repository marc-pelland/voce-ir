# Launch content (S60 Slice 5)

Draft artifacts for the v1.1.0 public launch. Everything here is a
**first draft** intended to be heavily edited before publishing —
the value is having something concrete to react to instead of a
blank page.

| File | Purpose | Length |
| --- | --- | --- |
| [`blog-post.md`](blog-post.md) | Long-form on `voce-ir.xyz/blog` | ~1900 words |
| [`show-hn.md`](show-hn.md) | Hacker News Show HN copy | ~500 words |
| [`x-thread.md`](x-thread.md) | 10-post X / Twitter thread | 10 posts |
| [`demo-video-shotlist.md`](demo-video-shotlist.md) | 60–75 sec demo video scenes | 7 scenes |

## How these fit together

The blog post is the **canonical case** — every other artifact
references it. The Show HN copy and the X thread are different
audiences pointing at the same blog post + repo. The demo video is
the lead media for all of them.

Sequencing the launch:

1. Blog post goes live at `voce-ir.xyz/blog/introducing-voce-ir`.
2. Demo video uploaded (YouTube or self-hosted) and embedded at the
   top of the blog post.
3. Show HN posted (Tue–Thu, 6:30–8:30 AM Pacific). Author present
   in comments for 90 min.
4. X thread posted ~2 hours after Show HN; pin tweet 1.
5. ProductHunt page goes live the morning after.

## Voice & tone notes (for the editor)

The project's existing prose style is technical, specific,
principled, and free of hype. Match that:

- Real numbers, not adjectives. ("52 validation rules, 17
  auto-fixable" beats "extensive validation.")
- Concrete examples beat abstract claims. Show the `aria-label`
  synthesis, the WCAG 2.2 contrast computation, the convergent fix
  loop's actual JSON output.
- The "anti-vibe-coding" framing is sharp on purpose. Don't soften
  it for marketing-friendliness, but be ready to clarify in replies
  that it describes *the workflow stance*, not other tools.
- "The code is gone. The experience remains." is the project's
  tagline; use it sparingly so it stays load-bearing rather than
  ornamental.

## What's deferred to the launch operator (not draftable here)

- The actual blog deployment to `voce-ir.xyz/blog/`
- Recording the demo video
- ProductHunt page setup
- Posting to HN / X / wherever
- Reply threads in real time

## Cross-references

- README discoverability (S60 Slice 1) — already shipped.
- The sprint plan: [`../plans/sprint-60-community-launch.md`](../plans/sprint-60-community-launch.md).
- Master roadmap context: [`../plans/MASTER_PLAN.md`](../plans/MASTER_PLAN.md).
