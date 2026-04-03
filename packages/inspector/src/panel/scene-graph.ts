/**
 * Scene graph tree view — hierarchical display of all IR nodes.
 */

import type { InspectedNode } from "../overlay/types.js";

export class SceneGraphTree {
  private selectedId: string | null = null;
  private container: HTMLElement | null = null;
  private onSelect: ((nodeId: string) => void) | null = null;

  /** Render the tree into a container element. */
  render(
    nodes: Map<string, InspectedNode>,
    container: HTMLElement,
    onSelect: (nodeId: string) => void
  ): void {
    this.container = container;
    this.onSelect = onSelect;

    container.innerHTML = `<div style="font-size:12px;font-family:monospace">
      <div style="color:#888;margin-bottom:8px">${nodes.size} nodes</div>
      ${this.buildTree(nodes)}
    </div>`;

    // Attach click handlers
    container.querySelectorAll<HTMLElement>("[data-tree-node]").forEach((el) => {
      el.addEventListener("click", (e) => {
        e.stopPropagation();
        const id = el.dataset.treeNode!;
        this.onSelect?.(id);
      });
    });
  }

  /** Highlight a node in the tree. */
  highlightNode(nodeId: string): void {
    if (!this.container) return;

    // Remove previous highlight
    this.container.querySelectorAll(".voce-tree-selected").forEach((el) => {
      (el as HTMLElement).style.backgroundColor = "";
      el.classList.remove("voce-tree-selected");
    });

    // Add new highlight
    const el = this.container.querySelector(`[data-tree-node="${nodeId}"]`);
    if (el) {
      (el as HTMLElement).style.backgroundColor = "rgba(68,136,255,0.15)";
      el.classList.add("voce-tree-selected");
    }

    this.selectedId = nodeId;
  }

  private buildTree(nodes: Map<string, InspectedNode>): string {
    const items: string[] = [];

    for (const [id, node] of nodes) {
      const typeColor = getTypeColor(node.type);
      const ariaInfo = node.aria.role ? ` [${node.aria.role}]` : "";

      items.push(
        `<div data-tree-node="${id}" style="padding:3px 6px;cursor:pointer;border-radius:3px;margin:1px 0" title="${id}">` +
          `<span style="color:${typeColor}">${node.type}</span> ` +
          `<span style="color:#666">${id}</span>` +
          `<span style="color:#888;font-size:11px">${ariaInfo}</span>` +
          `</div>`
      );
    }

    return items.join("");
  }
}

function getTypeColor(type: string): string {
  if (type.startsWith("Container")) return "#4488ff";
  if (type === "TextNode") return "#88cc44";
  if (type === "Surface" || type.startsWith("Surface")) return "#cc8844";
  if (type === "MediaNode") return "#cc44aa";
  if (type.startsWith("Semantic")) return "#ffaa22";
  if (type === "Scene3D") return "#44cccc";
  if (type === "FormNode") return "#aa44ff";
  return "#888";
}
