/**
 * Main inspector controller — manages overlay lifecycle, node discovery,
 * element highlighting, and keyboard shortcuts.
 */

import type { InspectedNode, InspectorState } from "./types.js";
import { SceneGraphTree } from "../panel/scene-graph.js";
import { PropertyPanel } from "../panel/properties.js";

export class VoceInspector {
  private state: InspectorState = {
    active: false,
    selectedNodeId: null,
    nodes: new Map(),
    panelVisible: false,
  };

  private overlay: HTMLDivElement | null = null;
  private highlighter: HTMLDivElement | null = null;
  private panel: HTMLDivElement | null = null;
  private sceneGraph: SceneGraphTree;
  private propertyPanel: PropertyPanel;

  constructor() {
    this.sceneGraph = new SceneGraphTree();
    this.propertyPanel = new PropertyPanel();
  }

  /** Initialize the inspector. Call once after page load. */
  static init(): VoceInspector {
    const inspector = new VoceInspector();
    inspector.setupKeyboardShortcut();
    return inspector;
  }

  /** Toggle the inspector on/off. */
  toggle(): void {
    if (this.state.active) {
      this.deactivate();
    } else {
      this.activate();
    }
  }

  /** Activate the inspector overlay. */
  activate(): void {
    if (this.state.active) return;
    this.state.active = true;

    // Discover all Voce IR nodes in the DOM
    this.discoverNodes();

    // Create overlay
    this.createOverlay();

    // Create panel
    this.createPanel();

    // Setup click-to-inspect
    this.setupClickHandler();
  }

  /** Deactivate and remove the inspector. */
  deactivate(): void {
    this.state.active = false;
    this.state.selectedNodeId = null;
    this.overlay?.remove();
    this.highlighter?.remove();
    this.panel?.remove();
    this.overlay = null;
    this.highlighter = null;
    this.panel = null;
  }

  /** Select a node by ID. */
  selectNode(nodeId: string): void {
    const node = this.state.nodes.get(nodeId);
    if (!node) return;

    this.state.selectedNodeId = nodeId;
    this.highlightElement(node.element);
    this.propertyPanel.showNode(node, this.panel!);
    this.sceneGraph.highlightNode(nodeId);
  }

  /** Discover all elements with data-voce-id attributes. */
  private discoverNodes(): void {
    this.state.nodes.clear();

    const elements = document.querySelectorAll<HTMLElement>("[data-voce-id]");
    for (const el of elements) {
      const id = el.dataset.voceId!;
      const node: InspectedNode = {
        id,
        element: el,
        type: inferNodeType(el),
        styles: extractComputedStyles(el),
        aria: extractAriaAttributes(el),
        children: findChildNodeIds(el),
        rect: el.getBoundingClientRect(),
      };
      this.state.nodes.set(id, node);
    }

    // Also discover elements with role attributes (SemanticNode targets)
    const roleElements = document.querySelectorAll<HTMLElement>("[role]");
    for (const el of roleElements) {
      if (!el.dataset.voceId) {
        const role = el.getAttribute("role")!;
        const id = `sem-${role}-${Math.random().toString(36).slice(2, 6)}`;
        const node: InspectedNode = {
          id,
          element: el,
          type: `SemanticNode(${role})`,
          styles: extractComputedStyles(el),
          aria: extractAriaAttributes(el),
          children: [],
          rect: el.getBoundingClientRect(),
        };
        this.state.nodes.set(id, node);
      }
    }
  }

  /** Create the translucent overlay. */
  private createOverlay(): void {
    this.overlay = document.createElement("div");
    this.overlay.id = "voce-inspector-overlay";
    this.overlay.style.cssText =
      "position:fixed;top:0;left:0;width:100%;height:100%;pointer-events:none;z-index:99998;";

    this.highlighter = document.createElement("div");
    this.highlighter.id = "voce-inspector-highlight";
    this.highlighter.style.cssText =
      "position:fixed;border:2px solid #4488ff;background:rgba(68,136,255,0.1);pointer-events:none;z-index:99999;display:none;transition:all 0.15s ease;";

    document.body.appendChild(this.overlay);
    document.body.appendChild(this.highlighter);
  }

  /** Create the side panel for node properties. */
  private createPanel(): void {
    this.panel = document.createElement("div");
    this.panel.id = "voce-inspector-panel";
    this.panel.style.cssText =
      "position:fixed;top:0;right:0;width:320px;height:100%;background:#1a1a2e;color:#e0e0e0;" +
      "font-family:system-ui;font-size:13px;overflow-y:auto;z-index:100000;box-shadow:-4px 0 20px rgba(0,0,0,0.3);" +
      "padding:16px;";
    this.panel.innerHTML = `
      <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:16px">
        <h3 style="margin:0;font-size:14px;color:#4488ff">Voce Inspector</h3>
        <button onclick="this.closest('#voce-inspector-panel').remove()" style="background:none;border:none;color:#888;cursor:pointer;font-size:18px">&times;</button>
      </div>
      <div id="voce-scene-graph"></div>
      <hr style="border:none;border-top:1px solid #333;margin:12px 0">
      <div id="voce-properties"></div>
    `;
    document.body.appendChild(this.panel);

    // Populate scene graph
    this.sceneGraph.render(
      this.state.nodes,
      document.getElementById("voce-scene-graph")!,
      (nodeId) => this.selectNode(nodeId)
    );

    this.state.panelVisible = true;
  }

  /** Setup click-to-inspect handler. */
  private setupClickHandler(): void {
    const handler = (e: MouseEvent) => {
      if (!this.state.active) return;
      if ((e.target as HTMLElement).closest("#voce-inspector-panel")) return;

      e.preventDefault();
      e.stopPropagation();

      const target = e.target as HTMLElement;
      const voceEl = target.closest<HTMLElement>("[data-voce-id]");
      if (voceEl) {
        this.selectNode(voceEl.dataset.voceId!);
      }
    };

    document.addEventListener("click", handler, true);
  }

  /** Highlight a DOM element with the blue overlay. */
  private highlightElement(element: HTMLElement): void {
    if (!this.highlighter) return;
    const rect = element.getBoundingClientRect();
    this.highlighter.style.display = "block";
    this.highlighter.style.top = `${rect.top}px`;
    this.highlighter.style.left = `${rect.left}px`;
    this.highlighter.style.width = `${rect.width}px`;
    this.highlighter.style.height = `${rect.height}px`;
  }

  /** Setup Ctrl+Shift+I keyboard shortcut. */
  private setupKeyboardShortcut(): void {
    document.addEventListener("keydown", (e) => {
      if (e.ctrlKey && e.shiftKey && e.key === "I") {
        e.preventDefault();
        this.toggle();
      }
    });
  }
}

/** Infer the IR node type from DOM element characteristics. */
function inferNodeType(el: HTMLElement): string {
  const tag = el.tagName.toLowerCase();
  if (tag === "img") return "MediaNode";
  if (tag === "form") return "FormNode";
  if (tag === "canvas") return "Scene3D";
  if (["h1", "h2", "h3", "h4", "h5", "h6", "p", "span"].includes(tag)) return "TextNode";
  if (el.style.display === "grid" || el.style.gridTemplateColumns) return "Container(Grid)";
  if (el.style.display === "flex") return "Container(Flex)";
  if (el.getAttribute("role") === "presentation") return "Surface(decorative)";
  return "Container";
}

/** Extract key computed styles. */
function extractComputedStyles(el: HTMLElement): Record<string, string> {
  const cs = getComputedStyle(el);
  const styles: Record<string, string> = {};
  const keys = [
    "display", "flexDirection", "justifyContent", "alignItems", "gap",
    "padding", "margin", "width", "height", "backgroundColor", "color",
    "fontSize", "fontWeight", "borderRadius", "boxShadow", "opacity",
  ];
  for (const key of keys) {
    const val = cs.getPropertyValue(key.replace(/([A-Z])/g, "-$1").toLowerCase());
    if (val && val !== "none" && val !== "normal" && val !== "0px") {
      styles[key] = val;
    }
  }
  return styles;
}

/** Extract ARIA attributes. */
function extractAriaAttributes(el: HTMLElement): Record<string, string> {
  const aria: Record<string, string> = {};
  for (const attr of el.attributes) {
    if (attr.name.startsWith("aria-") || attr.name === "role" || attr.name === "tabindex") {
      aria[attr.name] = attr.value;
    }
  }
  return aria;
}

/** Find child elements with data-voce-id. */
function findChildNodeIds(el: HTMLElement): string[] {
  const ids: string[] = [];
  for (const child of el.children) {
    const childEl = child as HTMLElement;
    if (childEl.dataset?.voceId) {
      ids.push(childEl.dataset.voceId);
    }
  }
  return ids;
}
