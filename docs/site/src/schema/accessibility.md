# Accessibility & Semantics Nodes

Accessibility in Voce IR is structurally required, not opt-in. Every interactive visual node must reference a SemanticNode. The validator rejects IR where interactive elements lack semantic annotations. Explicit opt-outs (`decorative: true`, `presentation: true`) are supported for valid exceptions.

## SemanticNode

Parallel semantic tree entry for a visual node. Carries ARIA role, label, relationships, and keyboard focus configuration. Visual nodes reference SemanticNodes via the `semantic_node_id` field. All SemanticNodes live in a flat list on ViewRoot.

| Field          | Type      | Required | Description                                       |
|----------------|-----------|----------|---------------------------------------------------|
| node_id        | string    | yes      | Unique identifier                                 |
| role           | string    | yes      | ARIA role (e.g., "button", "heading", "navigation", "main") |
| label          | string    | no       | Accessible label announced by screen readers      |
| labelled_by    | string    | no       | Node ID whose content labels this node (aria-labelledby) |
| described_by   | string    | no       | Node ID providing extended description (aria-describedby) |
| controls       | string    | no       | Node ID this element controls (aria-controls)     |
| owned_by       | string    | no       | Node ID that owns this element (aria-owns)        |
| heading_level  | int8      | no       | Heading level 1-6, only valid when role="heading" (default 0) |
| tab_index      | int32     | no       | Keyboard focus order (-2=unset, -1=programmatic, 0=natural, >0=explicit) |
| hidden         | bool      | no       | Hidden from the accessibility tree (default false)|
| aria_expanded  | int8      | no       | -1=unset, 0=false, 1=true                        |
| aria_selected  | int8      | no       | -1=unset, 0=false, 1=true                        |
| aria_checked   | int8      | no       | -1=unset, 0=false, 1=true, 2=mixed               |
| aria_disabled  | bool      | no       | Whether element is disabled (default false)       |
| aria_required  | bool      | no       | Whether element is required (default false)       |
| aria_invalid   | bool      | no       | Whether element has invalid input (default false) |
| aria_value_min | float32   | no       | Minimum value for range widgets                   |
| aria_value_max | float32   | no       | Maximum value for range widgets                   |
| aria_value_now | float32   | no       | Current value for range widgets                   |
| aria_value_text| string    | no       | Human-readable value for range widgets            |
| custom_aria    | [KeyValue]| no       | Custom ARIA attributes (escape hatch)             |

The validator enforces that interactive roles ("button", "link", "textbox", etc.) have either `label` or `labelled_by` set.

```json
{
  "node_id": "sem-cta-btn",
  "role": "button",
  "label": "Get started with Voce IR",
  "tab_index": 0
}
```

## LiveRegion

Declares a region whose content changes are announced by screen readers. Used for toast notifications, form errors, cart updates, and status messages.

| Field            | Type                   | Required | Description                                  |
|------------------|------------------------|----------|----------------------------------------------|
| node_id          | string                 | yes      | Unique identifier                            |
| target_node_id   | string                 | yes      | Visual node this live region is attached to   |
| politeness       | LiveRegionPoliteness   | no       | Polite (default) or Assertive or Off         |
| atomic           | bool                   | no       | Announce entire region on change (default false) |
| relevant         | LiveRegionRelevant     | no       | Additions (default), Removals, Text, All     |
| role_description | string                 | no       | Descriptive label (e.g., "Shopping cart updates") |

**Politeness values:**

- **Polite** -- wait for the user to be idle before announcing
- **Assertive** -- interrupt current speech to announce immediately
- **Off** -- region is silent

```json
{
  "node_id": "toast-live",
  "target_node_id": "toast-container",
  "politeness": "Assertive",
  "atomic": true,
  "role_description": "Notification"
}
```

## FocusTrap

Constrains keyboard focus to a subtree. Used for modals, drawers, and dialogs. The compiler emits focus management JavaScript.

| Field                  | Type             | Required | Description                                   |
|------------------------|------------------|----------|-----------------------------------------------|
| node_id                | string           | yes      | Unique identifier                             |
| container_node_id      | string           | yes      | Container node whose subtree traps focus      |
| initial_focus_node_id  | string           | no       | Node to focus on activation (default: first focusable) |
| escape_behavior        | FocusTrapEscape  | no       | CloseOnEscape (default), NoEscape, FireEvent  |
| escape_state_machine   | string           | no       | StateMachine for FireEvent escape behavior    |
| escape_event           | string           | no       | Event to fire on Escape for FireEvent         |
| restore_focus          | bool             | no       | Restore previous focus on deactivation (default true) |

```json
{
  "node_id": "modal-trap",
  "container_node_id": "modal-container",
  "initial_focus_node_id": "modal-close-btn",
  "escape_behavior": "CloseOnEscape",
  "restore_focus": true
}
```

## ReducedMotion

Every animation in the IR must reference a ReducedMotion alternative. The validator rejects any AnimationTransition, Sequence, or ScrollBinding that lacks one. See the [Motion](./motion.md) chapter for the full ReducedMotion reference.

**Strategy values:**

| Strategy       | Description                                                     |
|----------------|-----------------------------------------------------------------|
| Remove         | Remove animation entirely, snap to final state                  |
| Simplify       | Replace with a simpler animation (e.g., opacity fade)           |
| ReduceDuration | Keep animation but reduce to near-instant duration              |
| Functional     | Animation is functional (spinner, progress bar); simplify only  |
