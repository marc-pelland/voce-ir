# Voce IR MCP Server Setup

Use Voce IR tools directly from Claude Code (or any MCP-compatible client). Validate, compile, and inspect IR without leaving your conversation.

## Prerequisites

1. Build the Voce CLI:
   ```bash
   cd /path/to/voce-ir
   cargo build --release -p voce-validator
   ```

2. Build the MCP server:
   ```bash
   cd packages/mcp-server
   npm install
   npm run build
   ```

## Claude Code Setup

Add to your Claude Code MCP config. The config file location depends on your setup:

- **Claude Code CLI:** `~/.claude.json` or project-level `.mcp.json`
- **Claude Desktop:** `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS)

```json
{
  "mcpServers": {
    "voce-ir": {
      "command": "node",
      "args": ["/absolute/path/to/voce-ir/packages/mcp-server/dist/index.js"],
      "cwd": "/absolute/path/to/voce-ir"
    }
  }
}
```

Replace `/absolute/path/to/voce-ir` with the actual path to your voce-ir checkout.

The `cwd` field is important — the MCP server looks for the compiled `voce` binary in `target/release/` relative to the workspace root.

## Available Tools

Once connected, you can ask Claude to use these tools:

| Tool | What it does | Example prompt |
|------|-------------|----------------|
| `voce_validate` | Validate IR against 47 quality rules | "Validate this IR for me" |
| `voce_compile` | Compile IR to HTML | "Compile this to HTML" |
| `voce_inspect` | Get IR structure summary | "What's in this IR file?" |
| `voce_schema` | Look up node type documentation | "How does FormNode work?" |
| `voce_examples` | List/retrieve example IR files | "Show me the landing page example" |
| `voce_generate` | Generate IR from description (needs API key) | "Generate a contact form" |

## Example Conversation

```
You: Can you validate this IR?
{paste IR JSON}

Claude: [calls voce_validate] The IR has 0 errors and 1 warning:
- SEO007: OpenGraph data is present but missing og:image

You: Compile it to HTML

Claude: [calls voce_compile] Here's the compiled output (7.6KB):
<!DOCTYPE html>...

You: What node types does it use?

Claude: [calls voce_inspect] The IR contains 30 nodes:
- Container: 8
- TextNode: 15
- Surface: 6
- SemanticNode: 3
```

## Resources

The MCP server also exposes two resources:

- `voce://brief` — The project brief (`.voce/brief.yaml`)
- `voce://status` — Project health status

## Troubleshooting

**"Command failed" errors:** Make sure the voce binary is built (`cargo build --release -p voce-validator`) and the `cwd` in your MCP config points to the workspace root.

**Schema tool returns "not found":** Build the docs site first (`cd docs/site && mdbook build`).
