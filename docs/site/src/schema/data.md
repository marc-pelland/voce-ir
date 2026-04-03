# Data & Backend Nodes

The data layer covers mutations (ActionNode), real-time subscriptions (SubscriptionNode), authentication (AuthContextNode), CMS content (ContentSlot), and structured rich text (RichTextNode). Data reads are handled by DataNode in the [State](./state.md) chapter.

## ActionNode

Declares a server mutation with optimistic update strategy, cache invalidation, and error handling. The compiler emits mutation calls with rollback support.

| Field           | Type             | Required | Description                                     |
|-----------------|------------------|----------|-------------------------------------------------|
| node_id         | string           | yes      | Unique identifier                               |
| name            | string           | no       | Human-readable name                             |
| source          | DataSource       | yes      | Endpoint configuration                          |
| method          | HttpMethod       | no       | POST (default), GET, PUT, PATCH, DELETE         |
| input_type      | string           | no       | Input type hint for the compiler                |
| output_type     | string           | no       | Output type hint for the compiler               |
| optimistic      | OptimisticConfig | no       | Optimistic update behavior                      |
| invalidates     | [string]         | no       | DataNode IDs to refetch after success           |
| invalidate_tags | [string]         | no       | Cache tags to invalidate                        |
| error_handling  | ErrorHandling    | no       | Error and retry configuration                   |
| auth_required   | bool             | no       | Whether authentication is needed (default false)|
| required_roles  | [string]         | no       | Roles required to execute this action           |
| csrf_protected  | bool             | no       | CSRF protection enabled (default true)          |
| idempotent      | bool             | no       | Whether safe to retry (default false)           |

### OptimisticConfig

| Field               | Type               | Required | Description                              |
|---------------------|--------------------|----------|------------------------------------------|
| strategy            | OptimisticStrategy | no       | None (default), MirrorInput, CustomTransform |
| target_data_node_id | string             | no       | DataNode to optimistically update        |
| rollback            | RollbackStrategy   | no       | Revert (default) or ShowErrorKeepData    |

### ErrorHandling

| Field         | Type          | Required | Description                                  |
|---------------|---------------|----------|----------------------------------------------|
| retry         | RetryConfig   | no       | Retry configuration                          |
| fallback      | ErrorFallback | no       | ShowToast (default), ShowInlineError, Redirect |
| redirect_path | string        | no       | Path for Redirect fallback                   |

```json
{
  "node_id": "add-to-cart",
  "name": "add-item",
  "source": {
    "provider": "Rest",
    "endpoint": "/api/cart/items"
  },
  "method": "POST",
  "optimistic": {
    "strategy": "MirrorInput",
    "target_data_node_id": "cart-data",
    "rollback": "Revert"
  },
  "invalidates": ["cart-data"],
  "csrf_protected": true
}
```

## SubscriptionNode

Real-time data via WebSocket, Server-Sent Events, or polling. Keeps a DataNode updated with live changes.

| Field               | Type                    | Required | Description                                |
|---------------------|-------------------------|----------|--------------------------------------------|
| node_id             | string                  | yes      | Unique identifier                          |
| name                | string                  | no       | Human-readable name                        |
| source              | DataSource              | yes      | Endpoint configuration                     |
| transport           | SubscriptionTransport   | no       | WebSocket (default), ServerSentEvents, Polling |
| channel             | string                  | no       | Channel, topic, or table to subscribe to   |
| filter              | string                  | no       | Filter expression for the subscription     |
| update_strategy     | UpdateStrategy          | no       | Replace (default), Merge, Append           |
| target_data_node_id | string                  | yes      | DataNode to keep updated                   |
| connection          | ConnectionConfig        | no       | Reconnection and heartbeat settings        |

### ConnectionConfig

| Field                 | Type   | Required | Description                                 |
|-----------------------|--------|----------|---------------------------------------------|
| reconnect             | bool   | no       | Auto-reconnect on disconnect (default true) |
| reconnect_interval_ms | uint32 | no       | Reconnect interval in ms (default 3000)     |
| max_retries           | int32  | no       | Maximum reconnect attempts (default 10)     |
| heartbeat_interval_ms | uint32 | no       | Heartbeat interval in ms (default 30000)    |

```json
{
  "node_id": "chat-sub",
  "name": "chat-messages",
  "source": {
    "provider": "Supabase",
    "endpoint": "wss://project.supabase.co/realtime/v1"
  },
  "transport": "WebSocket",
  "channel": "messages",
  "update_strategy": "Append",
  "target_data_node_id": "messages-data"
}
```

## AuthContextNode

Application-level auth configuration. Provider-agnostic; the compiler adapts output to the chosen provider.

| Field             | Type            | Required | Description                                  |
|-------------------|-----------------|----------|----------------------------------------------|
| node_id           | string          | yes      | Unique identifier                            |
| name              | string          | no       | Human-readable name                          |
| provider          | AuthProvider    | no       | Auth0, Clerk, Supabase, Firebase, NextAuth, Custom |
| session_strategy  | SessionStrategy | no       | JwtCookie (default), JwtHeader, SessionCookie|
| user_type         | string          | no       | Type hint for the user object shape          |
| role_field        | string          | no       | Field path for roles in the user object      |
| login_action_id   | string          | no       | ActionNode reference for login               |
| logout_action_id  | string          | no       | ActionNode reference for logout              |
| refresh_action_id | string          | no       | ActionNode reference for token refresh       |

```json
{
  "node_id": "app-auth",
  "provider": "Supabase",
  "session_strategy": "JwtCookie",
  "role_field": "user_metadata.role",
  "login_action_id": "login-action",
  "logout_action_id": "logout-action"
}
```

## ContentSlot

Declares a CMS content dependency. The cache strategy determines compiler behavior: static content is baked in at build time, ISR adds revalidation, and dynamic content is fetched at runtime.

| Field          | Type                 | Required | Description                                |
|----------------|----------------------|----------|--------------------------------------------|
| node_id        | string               | yes      | Unique identifier                          |
| content_key    | string               | yes      | Content entry ID or path in the CMS        |
| source         | ContentSource        | no       | CMS provider and endpoint configuration    |
| fallback       | string               | no       | Fallback content if CMS fetch fails        |
| cache_strategy | ContentCacheStrategy | no       | Static (default), Isr, Dynamic             |
| content_type   | ContentType          | no       | Text (default), RichText, Media, Structured|
| locale         | string               | no       | Locale for this content slot               |

```json
{
  "node_id": "hero-content",
  "content_key": "homepage-hero",
  "source": {
    "provider": "sanity",
    "endpoint": "https://project.api.sanity.io/v2023-01-01"
  },
  "cache_strategy": "Isr",
  "content_type": "RichText"
}
```

## RichTextNode

Structured rich text with paragraphs, headings, lists, images, code blocks, and more. Maps directly from Sanity Portable Text, Contentful Rich Text JSON, and Payload Lexical JSON.

| Field        | Type            | Required | Description                        |
|--------------|-----------------|----------|------------------------------------|
| node_id      | string          | yes      | Unique identifier                  |
| blocks       | [RichTextBlock] | yes      | Ordered list of content blocks     |
| content_slot | ContentSlot     | no       | CMS source for the content         |

### RichTextBlock

| Field         | Type              | Required | Description                              |
|---------------|-------------------|----------|------------------------------------------|
| block_type    | RichTextBlockType | no       | Paragraph, Heading, UnorderedList, OrderedList, ListItem, Image, CodeBlock, Blockquote, Divider, Table, TableRow, TableCell |
| level         | int8              | no       | Heading level (1-6) or list depth        |
| children      | [RichTextSpan]    | no       | Inline text content                      |
| media_src     | string            | no       | Image source URL (for Image blocks)      |
| media_alt     | string            | no       | Image alt text (for Image blocks)        |
| code_language | string            | no       | Language hint (for CodeBlock blocks)     |
| rows          | [RichTextBlock]   | no       | Nested rows (for Table blocks)           |

### RichTextSpan

| Field    | Type            | Required | Description                          |
|----------|-----------------|----------|--------------------------------------|
| text     | string          | yes      | Text content                         |
| marks    | [RichTextMark]  | no       | Bold, Italic, Underline, Strikethrough, Code, Subscript, Superscript |
| link_url | string          | no       | URL if this span is a link           |

```json
{
  "node_id": "article-body",
  "blocks": [
    {
      "block_type": "Heading",
      "level": 2,
      "children": [{ "text": "Getting Started" }]
    },
    {
      "block_type": "Paragraph",
      "children": [
        { "text": "Voce IR uses " },
        { "text": "FlatBuffers", "marks": ["Bold"] },
        { "text": " for zero-copy deserialization." }
      ]
    }
  ]
}
```
