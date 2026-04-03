# Voce IR Examples

## Reference IR

- **[landing-page/](landing-page/)** — Hand-authored reference landing page (37 nodes, 11 types). Validates cleanly across all 9 passes. Used as the compilation benchmark target.

## Intent-IR Training Pairs

These pairs map natural language descriptions to complete IR output. Used for RAG retrieval and few-shot prompting in the AI bridge.

- **[01-hero-section/](intents/01-hero-section/)** — Hero with headline, subtitle, and CTA button
- **[02-contact-form/](intents/02-contact-form/)** — Contact form with name, email, message, and validation

## Demo Projects (Phase 3)

Complete projects built via AI conversation, demonstrating the full workflow.

- **[saas-landing/](demos/saas-landing/)** — SaaS landing page for "TaskFlow" (minimal-saas pack, 6-turn conversation)
- **[contact-form/](demos/contact-form/)** — Bakery contact/order page (ecommerce tones, includes incremental edit demo)
- **[marketing-site/](demos/marketing-site/)** — Architecture firm marketing site (editorial pack, 9 sections, hierarchical generation, voice interface turns)

Each demo includes:
- `conversation.txt` — Full conversation transcript
- `brief.yaml` — Generated project brief
- `output.voce.json` — Generated IR (when run with API)
- `index.html` — Compiled HTML output (when run with API)
