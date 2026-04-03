# Voce IR — Data Layer & Backend Integration

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how Voce IR handles data fetching, mutations, real-time, authentication, and CMS integration — the full backend story.

---

## 0. The Design Principle

Next.js server actions succeed because they hide the network boundary. The developer thinks "call this function" and the framework handles transport, serialization, error handling, and cache invalidation. Voce IR goes further: **the AI declares data intent in the IR, and the compiler emits the entire data pipeline** — fetching, caching, mutations, optimistic updates, error handling, auth, real-time, and content injection.

---

## 1. The Data Trinity: DataNode, ActionNode, SubscriptionNode

Every modern framework converges on three data primitives (the GraphQL trinity, generalized):

| IR Node | Purpose | Compiles To | Precedent |
| ------- | ------- | ----------- | --------- |
| **DataNode** | Declarative data fetching (reads) | TanStack Query `useQuery` / fetch + cache | Server Components, SWR, Relay queries |
| **ActionNode** | Declarative mutations (writes) | TanStack Query `useMutation` / fetch + optimistic update | Server Actions, Relay mutations |
| **SubscriptionNode** | Declarative real-time (live data) | WebSocket / SSE / polling subscription | Supabase Realtime, Convex reactive queries |

### DataNode (Expanded)

```
DataNode {
  id: string
  source: DataSource {
    provider: enum (rest, graphql, supabase, firebase, convex, custom)
    endpoint: string
    resource: string        // table name, collection, URL path
    operation: string       // select, query, rpc
  }
  query: QueryConfig {
    filters: [FilterExpr]
    sort: [SortExpr]
    pagination: PaginationConfig
    fields: [string]        // field selection (avoid over-fetching)
  }
  cache_strategy: CacheConfig {
    stale_time: Duration    // how long data is considered fresh
    cache_time: Duration    // how long to keep unused data
    invalidation_tags: [string]
    deduplication: bool     // merge identical concurrent requests
  }
  refresh_triggers: [RefreshTrigger]  // navigation, focus, interval, manual
  auth_required: bool
  loading_state: Ref<StateMachine>    // state machine for loading/error/success
  error_fallback: Ref<Container>      // UI to show on error
}
```

### ActionNode (The Server Action Equivalent)

This is the key innovation — declarative mutations that the compiler emits all plumbing for:

```
ActionNode {
  id: string
  endpoint: DataSource              // same provider model as DataNode
  method: HttpMethod                // POST, PUT, DELETE, PATCH
  input_type: TypeRef               // FlatBuffers type for input validation
  output_type: TypeRef              // FlatBuffers type for response
  optimistic: OptimisticConfig {
    strategy: enum (none, mirror_input, custom_transform)
    target_data_node: string        // which DataNode to optimistically update
    rollback: enum (revert, show_error_keep_data)
  }
  invalidates: [string]             // DataNode IDs to refetch on success
  error_handling: ErrorConfig {
    retry: RetryConfig { max_attempts: int, backoff: enum }
    fallback: enum (show_toast, show_inline_error, redirect)
  }
  auth_required: bool
  required_roles: [string]
  csrf_protected: bool              // default true for mutations
  idempotent: bool                  // safe to retry?
}
```

### SubscriptionNode

```
SubscriptionNode {
  id: string
  source: DataSource
  channel: string                   // topic/room/table
  transport: enum (websocket, sse, polling)
  filter: FilterExpr
  update_strategy: enum (replace, merge, append)
  target_data_node: string          // which DataNode to keep updated
  connection: ConnectionConfig {
    reconnect: bool
    reconnect_interval: Duration
    max_retries: int
    heartbeat_interval: Duration
  }
}
```

---

## 2. Authentication & Authorization

Auth is modeled as a **ContextNode** (globally available state) + **RouteMap guards** + **ActionNodes** for login/logout:

```
AuthContextNode {
  provider: enum (auth0, clerk, supabase, firebase, nextauth, custom)
  session_strategy: enum (jwt_cookie, jwt_header, session_cookie)
  user_schema: TypeRef              // FlatBuffers type for user object
  role_field: string                // which field contains roles
  login_action: Ref<ActionNode>
  logout_action: Ref<ActionNode>
  refresh_action: Ref<ActionNode>
}
```

Route guards on RouteMap:

```
RouteEntry {
  path: string
  view: Ref<ViewRoot>
  guard: RouteGuard {
    requires_auth: bool
    required_roles: [string]
    redirect_on_fail: string        // e.g., "/login"
  }
}
```

**The IR does NOT model specific auth providers.** It models abstract concepts (authenticated, roles, guard, login/logout). The compiler or runtime adapter connects to the actual provider. Switching from Auth0 to Clerk is a config change, not an IR rewrite.

---

## 3. Content Management (CMS Integration)

### ContentSlot — Declarative Content

```
ContentSlot {
  content_key: string               // CMS entry ID or content path
  source: ContentSource {
    provider: enum (contentful, sanity, strapi, payload, keystatic, custom)
    endpoint: string
    auth_method: enum (api_key, bearer, none)
  }
  fallback: string                  // default content if fetch fails
  cache_strategy: enum (static, isr, dynamic)
  locale: string                    // localization key
  content_type: enum (text, rich_text, media, structured)
}
```

Cache strategy determines compiler behavior:

| Strategy | Build-Time | Runtime | Use Case |
| -------- | ---------- | ------- | -------- |
| `static` | Fetch and inline into output | No fetch | Blog posts, marketing copy, stable content |
| `isr` | Fetch and inline, include revalidation mechanism | Background revalidation via webhook | Product descriptions, pricing, content updated occasionally |
| `dynamic` | Use fallback value | Fetch from CMS API | User-specific content, real-time feeds, comments |

### RichTextNode — Structured Rich Text

```
RichTextNode {
  blocks: [RichTextBlock]
}

RichTextBlock {
  type: enum (paragraph, heading, list, image, code, quote, divider)
  level: int8                       // heading level, list depth
  children: [RichTextSpan]
  media: MediaNode                  // for image/video blocks
}

RichTextSpan {
  text: string
  marks: [enum (bold, italic, underline, code, strikethrough)]
  link_url: string
}
```

Maps directly from Sanity Portable Text, Contentful Rich Text JSON, and Payload Lexical JSON. The compiler emits semantic HTML (`<h2>`, `<ul>`, `<blockquote>`) rather than `<div>` with CSS classes.

### Content Build Pipeline

```
CMS content change → webhook → build server
  → identify affected IR files (content key → IR file map)
  → fetch updated content from CMS API (~100-300ms)
  → inject into ContentSlot nodes (~10ms)
  → compile IR to output (~100ms, Rust)
  → upload to CDN (~100-500ms)
  → invalidate CDN cache (~100ms with Cloudflare)

Total: 400ms to 3s for content update to go live
```

### Visual Editing

Because the compiled DOM output maps elements back to IR nodes (via `data-voce-node-id` attributes), visual editing comes almost for free:

1. Compiled output includes data attributes for IR node mapping
2. Inspector bridge script (~5KB) detects edit mode
3. Click element → identify ContentSlot → show edit panel
4. Edit content → save to CMS → hot-patch DOM or trigger recompile
5. Publish → webhook → rebuild → deploy

**Voce IR advantage:** Frameworks like React require manual annotation of every component for visual editing. Voce IR's compiler generates the annotations automatically because the IR-to-DOM mapping is explicit.

---

## 4. Compiler Strategy for Data Layer

### Key Decision: Emit TanStack Query

The DOM compiler should emit TanStack Query calls rather than building custom cache management:

- `DataNode` → `useQuery({ queryKey, queryFn, staleTime, gcTime })`
- `ActionNode` → `useMutation({ mutationFn, onMutate, onError, onSettled })`
- `SubscriptionNode` → custom hook wrapping WebSocket/SSE + query invalidation

TanStack Query is ~13KB gzipped and handles: stale-while-revalidate, deduplication, background refetch, window focus refetch, retry with backoff, optimistic updates with rollback, and cache invalidation. Building this custom would be enormous effort and inevitably worse.

**This means the <10KB target applies only to pages without data fetching.** Pages with DataNodes/ActionNodes will include the TanStack Query runtime (~13KB). This is a fair tradeoff — you're adding a data management library, not a UI framework. Total for a data-driven page: ~20-25KB vs ~200KB+ for Next.js equivalent.

For pages without any data layer nodes (pure static content), the compiled output remains zero-dependency.

### Provider Adapters

The compiler includes thin adapters per provider:

- **Supabase adapter:** Emits `supabase.from('table').select()` calls
- **Firebase adapter:** Emits `collection(db, 'name').where()` calls
- **Generic REST adapter:** Emits `fetch(url, options)` calls
- **GraphQL adapter:** Emits query/mutation strings with fetch

Each adapter is a separate module in the compiler — adding a new provider doesn't touch existing code.

---

## 5. Backend-as-a-Service Recommendations

### Tier 1 (Best Fit for Voce IR)

**Supabase** — Open-source, Postgres-based, auto-generated REST API, standard JWT auth, WebSocket real-time. The most "compiler-friendly" backend because everything has a predictable REST API shape. RLS provides row-level security without custom middleware.

**Convex** — Reactive model aligns perfectly with declarative IR. Query functions ARE the data declarations; Convex handles real-time automatically. The most "magical" developer experience for real-time.

### Tier 2 (Good Fit)

**Firebase** — Massive adoption, great SDKs, real-time by default. Less flexible querying than Postgres.

**PocketBase** — Single-binary backend, perfect for prototyping and "generate a full app" demos.

### Essential: Custom REST/GraphQL Support

Not everyone uses a BaaS. The IR must support arbitrary REST/GraphQL endpoints via the generic adapter.

---

## 6. What's Essential vs Future

### Phase 1 (Schema Design — Now)
- Define DataNode, ActionNode, SubscriptionNode, ContentSlot, RichTextNode, AuthContextNode in FlatBuffers schema
- Validation passes for data completeness (DataNodes have error states, ActionNodes have auth annotations)
- Security validation (allowed origins, CSRF annotations)

### Phase 2 (DOM Compiler)
- DataNode → TanStack Query compilation (generic REST adapter first)
- ActionNode → mutation compilation with optimistic updates
- ContentSlot → build-time content injection (static strategy)
- Auth state → ContextNode compilation
- Route guards → compiled middleware

### Phase 3 (AI Bridge)
- AI generates DataNode/ActionNode/ContentSlot from natural language ("show products from my API")
- Supabase and Firebase adapter support
- ISR-style content revalidation
- SubscriptionNode → real-time compilation

### Phase 4+ (Advanced)
- Convex reactive adapter
- Visual editing integration (inspector + CMS bridge)
- Server-side code generation (compile ActionNodes to edge function handlers)
- Offline support with sync

---

## 7. Reference Architectures

- **Relay** — Compiler-driven data fetching with fragment colocation. Closest architectural precedent for "declare data dependencies, compiler optimizes." Study its normalized cache and optimistic update model.
- **Next.js Server Actions** — The UX target. The feeling of "just call a function" that hides the network boundary.
- **TanStack Query** — The runtime target for the DOM compiler. Don't rebuild cache management; emit TanStack Query calls.
- **Supabase** — The default BaaS recommendation. Open-source, Postgres, predictable APIs.
- **Sanity Portable Text** — The structured rich text format to target first for CMS content adapters.

---

*This document should be read alongside `DEEP_RESEARCH.md` and `SECURITY_TESTING_TOOLING.md` for the full research context.*
