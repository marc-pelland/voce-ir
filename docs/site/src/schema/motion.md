# Motion & Interaction Nodes

Animation is a first-class IR concern in Voce IR. Every motion declaration includes a reduced-motion fallback, which the validator enforces as required. The compiler uses a tiered output strategy: CSS transitions, Web Animations API, then minimal requestAnimationFrame JavaScript.

## AnimationTransition

Animates property changes between state machine states. The compiler chooses the optimal output technique based on the trigger type and interruptibility requirements.

| Field                  | Type               | Required | Description                                      |
|------------------------|--------------------|----------|--------------------------------------------------|
| node_id                | string             | yes      | Unique identifier                                |
| name                   | string             | no       | Human-readable name                              |
| target_node_id         | string             | yes      | Node to animate                                  |
| trigger_state_machine  | string             | no       | StateMachine that triggers this animation        |
| trigger_event          | string             | no       | Event on the state machine that starts animation |
| properties             | [AnimatedProperty] | yes      | Properties to animate                            |
| duration               | Duration           | no       | Animation duration in ms                         |
| delay                  | Duration           | no       | Delay before animation starts                    |
| easing                 | Easing             | no       | Timing function (supports Spring)                |
| reduced_motion         | ReducedMotion      | no       | Required alternative for prefers-reduced-motion  |

### AnimatedProperty

| Field    | Type   | Required | Description                                        |
|----------|--------|----------|----------------------------------------------------|
| property | string | yes      | CSS-like property path (e.g., "opacity", "transform.translateY") |
| from     | string | yes      | Value in the starting state                        |
| to       | string | yes      | Value in the ending state                          |

### Easing

The `Easing` table supports multiple timing function types.

| Field        | Type       | Required | Description                                  |
|--------------|------------|----------|----------------------------------------------|
| easing_type  | EasingType | no       | Linear (default), CubicBezier, Spring, Steps, CustomLinear |
| x1, y1, x2, y2 | float32 | no      | Control points for CubicBezier               |
| stiffness    | float32    | no       | Spring stiffness (default 200)               |
| damping      | float32    | no       | Spring damping (default 20)                  |
| mass         | float32    | no       | Spring mass (default 1)                      |
| steps        | int32      | no       | Step count for Steps easing                  |
| points       | [float32]  | no       | Pre-computed points for CustomLinear (CSS linear()) |

```json
{
  "node_id": "fade-in",
  "target_node_id": "hero-section",
  "trigger_state_machine": "page-state",
  "trigger_event": "enter",
  "properties": [
    { "property": "opacity", "from": "0", "to": "1" },
    { "property": "transform.translateY", "from": "20px", "to": "0px" }
  ],
  "duration": { "ms": 300 },
  "easing": { "easing_type": "Spring", "stiffness": 300, "damping": 25, "mass": 1 },
  "reduced_motion": { "strategy": "Remove" }
}
```

## Sequence

Choreographed animation timeline. Multiple AnimationTransitions played in sequence or parallel with stagger offsets.

| Field      | Type            | Required | Description                                    |
|------------|-----------------|----------|------------------------------------------------|
| node_id    | string          | yes      | Unique identifier                              |
| name       | string          | no       | Human-readable name                            |
| steps      | [SequenceStep]  | yes      | Ordered list of animation steps                |
| stagger    | Duration        | no       | Delay between sequential elements              |
| iterations | int32           | no       | Repeat count, 0 = infinite (default 1)         |
| alternate  | bool            | no       | Reverse on alternate iterations (default false)|
| reduced_motion | ReducedMotion | no     | Alternative for the entire sequence            |

### SequenceStep

| Field         | Type     | Required | Description                                     |
|---------------|----------|----------|-------------------------------------------------|
| transition_id | string   | yes      | Reference to an AnimationTransition node         |
| delay         | Duration | no       | Delay before this step starts                    |
| parallel      | bool     | no       | If true, runs concurrently with previous step (default false) |

```json
{
  "node_id": "entrance-seq",
  "steps": [
    { "transition_id": "fade-in-title" },
    { "transition_id": "fade-in-subtitle", "delay": { "ms": 100 } },
    { "transition_id": "fade-in-cta", "delay": { "ms": 100 } }
  ],
  "stagger": { "ms": 50 },
  "reduced_motion": { "strategy": "Remove" }
}
```

## GestureHandler

Maps touch, mouse, and keyboard input to state transitions or continuous property updates. The validator requires a `keyboard_key` for accessibility.

| Field                  | Type        | Required | Description                                    |
|------------------------|-------------|----------|------------------------------------------------|
| node_id                | string      | yes      | Unique identifier                              |
| target_node_id         | string      | yes      | Node that receives the gesture                 |
| gesture_type           | GestureType | no       | Tap, DoubleTap, LongPress, Drag, Swipe, Pinch, Hover, Focus |
| trigger_event          | string      | no       | State machine event to fire on gesture         |
| trigger_state_machine  | string      | no       | Target state machine                           |
| continuous_property    | string      | no       | Property to update for drag/continuous gestures|
| continuous_axis        | string      | no       | Axis for continuous gestures                   |
| keyboard_key           | string      | no       | Keyboard equivalent (required by validator)    |
| keyboard_modifier      | string      | no       | Modifier key (Shift, Ctrl, Alt, Meta)          |
| threshold_px           | float32     | no       | Gesture distance threshold in pixels           |
| velocity_threshold     | float32     | no       | Gesture velocity threshold                     |

```json
{
  "node_id": "card-tap",
  "target_node_id": "card-surface",
  "gesture_type": "Tap",
  "trigger_event": "select",
  "trigger_state_machine": "card-state",
  "keyboard_key": "Enter"
}
```

## ScrollBinding

Binds node properties to scroll position. Compiled to CSS scroll-driven animations where supported, with IntersectionObserver fallback.

| Field               | Type               | Required | Description                                  |
|---------------------|--------------------|----------|----------------------------------------------|
| node_id             | string             | yes      | Unique identifier                            |
| target_node_id      | string             | yes      | Node whose properties are scroll-linked      |
| scroll_trigger      | ScrollTrigger      | no       | ViewProgress (default) or ScrollProgress     |
| scroll_axis         | ScrollAxis         | no       | Vertical (default) or Horizontal             |
| range_start         | float32            | no       | Start of scroll range, 0.0-1.0 (default 0.0)|
| range_end           | float32            | no       | End of scroll range, 0.0-1.0 (default 1.0)  |
| properties          | [AnimatedProperty] | yes      | Properties to animate over the scroll range  |
| scroll_container_id | string             | no       | Scroll container (default: nearest ancestor) |
| reduced_motion      | ReducedMotion      | no       | Alternative for prefers-reduced-motion       |

```json
{
  "node_id": "parallax-bg",
  "target_node_id": "bg-image",
  "scroll_trigger": "ViewProgress",
  "properties": [
    { "property": "transform.translateY", "from": "0px", "to": "-50px" }
  ],
  "reduced_motion": { "strategy": "Remove" }
}
```

## PhysicsBody

Attaches physics simulation to a node for spring animations, momentum scrolling, and procedural motion. Non-interruptible springs are compiled to CSS `linear()` at build time.

| Field          | Type    | Required | Description                                     |
|----------------|---------|----------|-------------------------------------------------|
| node_id        | string  | yes      | Unique identifier                               |
| target_node_id | string  | yes      | Node to apply physics to                        |
| stiffness      | float32 | no       | Spring stiffness (default 300)                  |
| damping        | float32 | no       | Spring damping (default 25, must be > 0)        |
| mass           | float32 | no       | Spring mass (default 1)                         |
| friction       | float32 | no       | Momentum friction, 0-1 (default 0.05)           |
| restitution    | float32 | no       | Bounciness, 0-1 (default 0)                     |
| interruptible  | bool    | no       | If true, uses rAF instead of CSS (default false)|

```json
{
  "node_id": "spring-body",
  "target_node_id": "draggable-card",
  "stiffness": 400,
  "damping": 30,
  "mass": 1,
  "interruptible": true
}
```

## ReducedMotion

Every animation must reference a ReducedMotion alternative. The validator rejects IR where any AnimationTransition, Sequence, or ScrollBinding lacks one.

| Field                 | Type                    | Required | Description                              |
|-----------------------|-------------------------|----------|------------------------------------------|
| strategy              | ReducedMotionStrategy   | no       | Remove (default), Simplify, ReduceDuration, Functional |
| simplified_properties | [AnimatedProperty]      | no       | Replacement properties for Simplify strategy |
| reduced_duration      | Duration                | no       | Shortened duration for ReduceDuration strategy |

**Strategy values:**

- **Remove** -- snap to final state, no animation
- **Simplify** -- replace with a simpler animation (e.g., fade instead of slide)
- **ReduceDuration** -- keep the animation but make it near-instant
- **Functional** -- animation serves a functional purpose (spinner, progress); simplify but do not remove
