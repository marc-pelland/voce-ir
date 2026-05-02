// Shared shape for tool results. Matches the MCP CallTool result wire
// format so an executor's return value can be handed straight to the
// transport without translation. Both the MCP server and the cli-chat
// tool-use loop consume this type.

export interface ToolResult {
  content: Array<{ type: "text"; text: string }>;
  isError?: boolean;
}

export interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: {
    type: "object";
    properties: Record<string, unknown>;
    required?: string[];
  };
}
