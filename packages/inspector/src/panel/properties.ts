/**
 * Property panel — shows IR node properties for the selected element.
 */

import type { InspectedNode } from "../overlay/types.js";

export class PropertyPanel {
  /** Show properties for a selected node. */
  showNode(node: InspectedNode, panelContainer: HTMLElement): void {
    const propsEl = panelContainer.querySelector("#voce-properties");
    if (!propsEl) return;

    propsEl.innerHTML = this.renderNode(node);
  }

  private renderNode(node: InspectedNode): string {
    return `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#4488ff;margin-bottom:4px">${node.type}</div>
        <div style="color:#888;font-size:11px">ID: ${node.id}</div>
        <div style="color:#888;font-size:11px">Size: ${Math.round(node.rect.width)}×${Math.round(node.rect.height)}</div>
      </div>

      ${this.renderSection("ARIA / Accessibility", node.aria)}
      ${this.renderSection("Computed Styles", node.styles)}
      ${node.children.length > 0 ? this.renderList("Children", node.children) : ""}
    `;
  }

  private renderSection(title: string, data: Record<string, string>): string {
    const entries = Object.entries(data);
    if (entries.length === 0) return "";

    return `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:4px">${title}</div>
        ${entries
          .map(
            ([key, value]) =>
              `<div style="display:flex;gap:8px;padding:2px 0;border-bottom:1px solid #222">
                <span style="color:#888;min-width:120px">${key}</span>
                <span style="color:#ddd;word-break:break-all">${escapeHtml(value)}</span>
              </div>`
          )
          .join("")}
      </div>
    `;
  }

  private renderList(title: string, items: string[]): string {
    return `
      <div style="margin-bottom:12px">
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:4px">${title}</div>
        ${items.map((id) => `<div style="color:#4488ff;padding:2px 0;cursor:pointer">${id}</div>`).join("")}
      </div>
    `;
  }
}

function escapeHtml(str: string): string {
  return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
