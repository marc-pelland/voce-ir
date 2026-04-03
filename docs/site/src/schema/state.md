# State & Logic Nodes

All state in Voce IR is modeled as explicit, typed finite state machines. There are no implicit closures, dependency arrays, or hook ordering. The compiler can statically analyze every possible state transition.

## StateMachine

A named finite state machine with typed states, transitions, guards, and effects. Every component's behavior is a state machine. The validator checks reachability (no dead states) and deadlock freedom.

### State

| Field    | Type   | Required | Description                                    |
|----------|--------|----------|------------------------------------------------|
| name     | string | yes      | State name                                     |
| initial  | bool   | no       | Whether this is the initial state (default false) |
| terminal | bool   | no       | Whether this is a final state (default false)  |

### Transition

| Field  | Type   | Required | Description                                       |
|--------|--------|----------|---------------------------------------------------|
| event  | string | yes      | Event name that triggers this transition           |
| from   | string | yes      | Source state name                                  |
| to     | string | yes      | Target state name                                  |
| guard  | string | no       | Reference to a ComputeNode that returns bool       |
| effect | string | no       | Reference to an EffectNode to execute on transition|

### StateMachine

| Field       | Type          | Required | Description                              |
|-------------|---------------|----------|------------------------------------------|
| node_id     | string        | yes      | Unique identifier                        |
| name        | string        | no       | Human-readable name (e.g., "auth-flow")  |
| states      | [State]       | yes      | List of states                           |
| transitions | [Transition]  | yes      | List of transitions                      |

```json
{
  "node_id": "btn-state",
  "name": "button-state",
  "states": [
    { "name": "idle", "initial": true },
    { "name": "hovered" },
    { "name": "pressed" },
    { "name": "disabled", "terminal": true }
  ],
  "transitions": [
    { "event": "hover", "from": "idle", "to": "hovered" },
    { "event": "unhover", "from": "hovered", "to": "idle" },
    { "event": "press", "from": "hovered", "to": "pressed" },
    { "event": "release", "from": "pressed", "to": "idle", "effect": "submit-effect" }
  ]
}
```

## DataNode

Declares an external data dependency. The compiler emits fetch code with caching, error handling, and loading states.

| Field                 | Type           | Required | Description                                        |
|-----------------------|----------------|----------|----------------------------------------------------|
| node_id               | string         | yes      | Unique identifier                                  |
| name                  | string         | no       | Human-readable name                                |
| source                | DataSource     | yes      | Endpoint configuration                             |
| cache_strategy        | CacheStrategy  | no       | None, StaleWhileRevalidate (default), CacheUntilInvalidated, Static |
| stale_time            | uint32         | no       | Freshness duration in ms (default 30000)           |
| cache_time            | uint32         | no       | Cache retention in ms (default 300000)             |
| cache_tags            | [string]       | no       | Tags for cache invalidation                        |
| auth_required         | bool           | no       | Whether authentication is needed (default false)   |
| loading_state_machine | string         | no       | StateMachine tracking loading/error/success         |
| query_params          | [KeyValue]     | no       | Query parameters for filtering, sorting, pagination|

### DataSource

| Field    | Type               | Required | Description                              |
|----------|--------------------|----------|------------------------------------------|
| provider | DataSourceProvider | no       | Rest (default), GraphQL, Supabase, Firebase, Convex, Custom |
| endpoint | string             | no       | API endpoint URL                         |
| resource | string             | no       | Resource path or query                   |
| method   | HttpMethod         | no       | GET (default), POST, PUT, PATCH, DELETE  |
| headers  | [KeyValue]         | no       | Custom request headers                   |

```json
{
  "node_id": "user-data",
  "name": "current-user",
  "source": {
    "provider": "Rest",
    "endpoint": "/api/users/me",
    "method": "GET"
  },
  "cache_strategy": "StaleWhileRevalidate",
  "stale_time": 60000,
  "auth_required": true
}
```

## ComputeNode

A pure function from inputs to output. Referentially transparent, so the compiler can memoize or pre-compute at build time.

| Field       | Type           | Required | Description                              |
|-------------|----------------|----------|------------------------------------------|
| node_id     | string         | yes      | Unique identifier                        |
| name        | string         | no       | Human-readable name                      |
| inputs      | [ComputeInput] | yes      | Input bindings from other nodes          |
| expression  | string         | yes      | Expression to evaluate                   |
| output_type | string         | no       | Output type hint for the compiler        |

### ComputeInput

| Field          | Type   | Required | Description                                    |
|----------------|--------|----------|------------------------------------------------|
| name           | string | yes      | Parameter name in the expression               |
| source_node_id | string | yes      | Source DataNode, ContextNode, or ComputeNode   |
| field_path     | string | no       | Dot-path into the source data                  |

```json
{
  "node_id": "total-compute",
  "name": "order-total",
  "inputs": [
    { "name": "price", "source_node_id": "product-data", "field_path": "price" },
    { "name": "qty", "source_node_id": "cart-ctx", "field_path": "quantity" }
  ],
  "expression": "price * qty",
  "output_type": "number"
}
```

## EffectNode

A side effect triggered by a state transition. Effects attach to transitions, never to states, eliminating mount/unmount ambiguity.

| Field       | Type       | Required | Description                                     |
|-------------|------------|----------|-------------------------------------------------|
| node_id     | string     | yes      | Unique identifier                               |
| name        | string     | no       | Human-readable name                             |
| effect_type | EffectType | no       | Analytics, ApiCall, Storage, Haptic, Log, Navigate, Custom |
| config      | [KeyValue] | no       | Configuration payload                           |
| api_source  | DataSource | no       | Endpoint configuration for ApiCall effects      |
| idempotent  | bool       | no       | Whether safe to retry (default false)           |

```json
{
  "node_id": "track-click",
  "name": "analytics-click",
  "effect_type": "Analytics",
  "config": [
    { "key": "event", "value": "button_click" },
    { "key": "category", "value": "cta" }
  ]
}
```

## ContextNode

Shared state scoped to a subtree. Replaces React Context, Redux, and prop drilling. Typed with explicit read/write boundaries.

| Field         | Type     | Required | Description                                      |
|---------------|----------|----------|--------------------------------------------------|
| node_id       | string   | yes      | Unique identifier                                |
| name          | string   | yes      | Context name                                     |
| initial_value | string   | no       | Initial value as JSON string                     |
| writers       | [string] | no       | Node IDs allowed to write (empty = any descendant)|
| global        | bool     | no       | If true, not scoped to subtree (default false)   |

```json
{
  "node_id": "cart-ctx",
  "name": "shopping-cart",
  "initial_value": "{ \"items\": [], \"quantity\": 0 }",
  "global": true
}
```
