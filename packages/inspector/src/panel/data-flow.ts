/**
 * Data flow monitor — shows live DataNode values and binding updates.
 */

export interface DataNodeInfo {
  id: string;
  name: string;
  endpoint: string;
  status: "idle" | "loading" | "loaded" | "error";
  lastFetched: number | null;
  cacheStrategy: string;
  value: unknown;
}

export class DataFlowMonitor {
  private dataNodes: DataNodeInfo[] = [];

  /** Render the data flow view. */
  render(nodes: DataNodeInfo[], container: HTMLElement): void {
    this.dataNodes = nodes;

    container.innerHTML = `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          Data Flow (${nodes.length} sources)
        </div>
        ${nodes.length === 0 ? '<div style="color:#555">No DataNodes in this page</div>' : ""}
        ${nodes.map((n) => this.renderDataNode(n)).join("")}
      </div>
    `;
  }

  /** Update a specific DataNode's status. */
  updateStatus(nodeId: string, status: DataNodeInfo["status"], value?: unknown): void {
    const node = this.dataNodes.find((n) => n.id === nodeId);
    if (node) {
      node.status = status;
      if (value !== undefined) node.value = value;
      if (status === "loaded") node.lastFetched = Date.now();
    }
  }

  private renderDataNode(node: DataNodeInfo): string {
    const statusColor =
      node.status === "loaded" ? "#44ff44" :
      node.status === "loading" ? "#ffaa22" :
      node.status === "error" ? "#ff4444" : "#555";

    const statusIcon =
      node.status === "loaded" ? "✓" :
      node.status === "loading" ? "⏳" :
      node.status === "error" ? "✗" : "○";

    const lastFetched = node.lastFetched
      ? new Date(node.lastFetched).toLocaleTimeString()
      : "never";

    return `
      <div style="background:rgba(255,255,255,0.03);border:1px solid #333;border-radius:4px;padding:8px;margin-bottom:4px">
        <div style="display:flex;justify-content:space-between;align-items:center">
          <span style="color:#ddd;font-size:12px">${node.name || node.id}</span>
          <span style="color:${statusColor};font-size:11px">${statusIcon} ${node.status}</span>
        </div>
        <div style="color:#666;font-size:10px;margin-top:4px">
          ${node.endpoint} | Cache: ${node.cacheStrategy} | Last: ${lastFetched}
        </div>
      </div>
    `;
  }
}
