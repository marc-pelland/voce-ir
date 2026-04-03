# Navigation & Routing Nodes

Routing in Voce IR is modeled as a state machine where states are views and transitions are navigation events. The schema supports nested routes, guards for auth checks, data preloading, and sitemap generation.

## RouteMap

Top-level routing configuration for a multi-route application. Referenced from `VoceDocument.routes`.

| Field              | Type                  | Required | Description                                  |
|--------------------|-----------------------|----------|----------------------------------------------|
| node_id            | string                | yes      | Unique identifier                            |
| routes             | [RouteEntry]          | yes      | List of route definitions                    |
| not_found_route    | string                | no       | Route name or path for 404 pages             |
| default_transition | RouteTransitionConfig | no       | Default transition animation between routes  |

```json
{
  "node_id": "app-routes",
  "routes": [
    { "path": "/", "name": "home", "view_root_id": "home-root" },
    { "path": "/about", "name": "about", "view_root_id": "about-root" }
  ],
  "not_found_route": "/404"
}
```

## RouteEntry

Defines a single route mapping a URL path to a ViewRoot.

| Field                | Type                  | Required | Description                                   |
|----------------------|-----------------------|----------|-----------------------------------------------|
| path                 | string                | yes      | URL path pattern (e.g., "/products/:id")      |
| name                 | string                | no       | Route name for programmatic navigation        |
| view_root_id         | string                | yes      | ViewRoot to render for this route             |
| guard                | RouteGuard            | no       | Access control configuration                  |
| preload_data         | [string]              | no       | DataNode IDs to prefetch on navigation        |
| transition           | RouteTransitionConfig | no       | Transition animation for this route           |
| sitemap_priority     | float32               | no       | Sitemap priority 0.0-1.0 (default 0.5)       |
| sitemap_change_freq  | ChangeFrequency       | no       | Always, Hourly, Daily, Weekly, Monthly, Yearly, Never |
| sitemap_last_modified| string                | no       | Last modification date (ISO 8601)             |
| exclude_from_sitemap | bool                  | no       | Exclude from generated sitemap (default false)|
| children             | [RouteEntry]          | no       | Nested child routes                           |

```json
{
  "path": "/dashboard",
  "name": "dashboard",
  "view_root_id": "dashboard-root",
  "guard": {
    "requires_auth": true,
    "required_roles": ["user"],
    "redirect_on_fail": "/login"
  },
  "preload_data": ["user-data", "dashboard-stats"],
  "sitemap_priority": 0.3,
  "exclude_from_sitemap": true
}
```

## RouteGuard

Access control for a route. The compiler emits authentication and authorization checks before rendering.

| Field            | Type     | Required | Description                                     |
|------------------|----------|----------|-------------------------------------------------|
| requires_auth    | bool     | no       | Whether authentication is required (default false)|
| required_roles   | [string] | no       | Roles the user must have to access the route    |
| redirect_on_fail | string   | no       | Path to redirect to if guard fails              |
| custom_guard     | string   | no       | Reference to a ComputeNode for custom logic     |

```json
{
  "requires_auth": true,
  "required_roles": ["admin"],
  "redirect_on_fail": "/login"
}
```

## RouteTransitionConfig

Configures the animation played when navigating between routes.

| Field              | Type                 | Required | Description                                    |
|--------------------|----------------------|----------|------------------------------------------------|
| transition_type    | RouteTransitionType  | no       | None, Crossfade, Slide, SharedElement, Custom  |
| duration           | Duration             | no       | Transition duration in ms                      |
| easing             | Easing               | no       | Timing function                                |
| slide_direction    | SlideDirection       | no       | Left, Right, Up, Down (for Slide type)         |
| shared_elements    | [SharedElementPair]  | no       | Paired elements for SharedElement transitions  |
| custom_sequence_id | string               | no       | Reference to a Sequence node (for Custom type) |
| reduced_motion     | ReducedMotion        | no       | Alternative for prefers-reduced-motion         |

### SharedElementPair

| Field           | Type   | Required | Description                           |
|-----------------|--------|----------|---------------------------------------|
| transition_name | string | yes      | Identifier for the shared transition  |
| source_node_id  | string | no       | Node in the source route              |
| target_node_id  | string | no       | Node in the target route              |

```json
{
  "transition_type": "Crossfade",
  "duration": { "ms": 200 },
  "easing": { "easing_type": "CubicBezier", "x1": 0.4, "y1": 0, "x2": 0.2, "y2": 1 },
  "reduced_motion": { "strategy": "Remove" }
}
```
