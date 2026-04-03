/**
 * Accessibility tree viewer — mirrors the browser a11y tree showing
 * ARIA roles, labels, states, and focus order.
 */

export interface A11yNode {
  /** Element reference. */
  element: HTMLElement;
  /** ARIA role (or inferred role from tag). */
  role: string;
  /** Accessible name (aria-label, visible text, or alt). */
  name: string;
  /** ARIA states (expanded, selected, checked, etc). */
  states: Record<string, string>;
  /** Tab index (-1 = not in tab order, 0+ = in order). */
  tabIndex: number;
  /** Whether this node is missing a semantic annotation. */
  missingSemantic: boolean;
  /** Focus order position (1-based, or 0 if not focusable). */
  focusOrder: number;
}

export class A11yTreeViewer {
  /** Scan the page and build the accessibility tree. */
  scan(): A11yNode[] {
    const nodes: A11yNode[] = [];
    let focusIndex = 0;

    // Get all focusable and semantic elements
    const elements = document.querySelectorAll<HTMLElement>(
      "[role], [aria-label], [tabindex], a, button, input, select, textarea, [data-voce-id]"
    );

    for (const el of elements) {
      const role = el.getAttribute("role") || inferRole(el);
      const name = getAccessibleName(el);
      const states = getAriaStates(el);
      const tabIdx = el.tabIndex;
      const isFocusable = tabIdx >= 0;

      // Check if interactive element is missing semantic annotation
      const isInteractive = ["button", "link", "textbox", "checkbox", "radio", "combobox"].includes(role);
      const hasLabel = !!name;
      const missingSemantic = isInteractive && !hasLabel;

      nodes.push({
        element: el,
        role,
        name,
        states,
        tabIndex: tabIdx,
        missingSemantic,
        focusOrder: isFocusable ? ++focusIndex : 0,
      });
    }

    return nodes;
  }

  /** Render the a11y tree into a container. */
  render(nodes: A11yNode[], container: HTMLElement): void {
    const warnings = nodes.filter((n) => n.missingSemantic);

    container.innerHTML = `
      <div>
        <div style="font-weight:bold;color:#aaa;font-size:11px;text-transform:uppercase;margin-bottom:8px">
          Accessibility Tree (${nodes.length} nodes)
        </div>

        ${warnings.length > 0 ? `<div style="background:rgba(255,68,68,0.1);border:1px solid #ff4444;border-radius:4px;padding:8px;margin-bottom:8px;font-size:11px;color:#ff8888">
          ⚠ ${warnings.length} interactive element${warnings.length > 1 ? "s" : ""} missing accessible name
        </div>` : ""}

        ${nodes.map((n) => this.renderNode(n)).join("")}
      </div>
    `;
  }

  /** Render focus order overlay numbers on the page. */
  renderFocusOverlay(nodes: A11yNode[]): HTMLElement[] {
    const overlays: HTMLElement[] = [];

    for (const node of nodes) {
      if (node.focusOrder === 0) continue;

      const rect = node.element.getBoundingClientRect();
      const badge = document.createElement("div");
      badge.className = "voce-focus-badge";
      badge.textContent = String(node.focusOrder);
      badge.style.cssText = `position:fixed;top:${rect.top - 8}px;left:${rect.left - 8}px;` +
        "width:20px;height:20px;border-radius:50%;background:#4488ff;color:#fff;" +
        "font-size:10px;display:flex;align-items:center;justify-content:center;" +
        "font-weight:bold;z-index:99997;pointer-events:none;font-family:system-ui;";
      document.body.appendChild(badge);
      overlays.push(badge);
    }

    return overlays;
  }

  /** Generate a screen reader text preview. */
  screenReaderPreview(nodes: A11yNode[]): string {
    const lines: string[] = [];

    for (const node of nodes) {
      if (node.role === "presentation" || node.role === "none") continue;

      let line = "";
      if (node.role === "heading") {
        const level = node.element.tagName.match(/H(\d)/)?.[1] || "?";
        line = `[heading level ${level}] ${node.name}`;
      } else if (node.role === "button") {
        line = `[button] ${node.name}`;
      } else if (node.role === "link") {
        line = `[link] ${node.name}`;
      } else if (node.role === "navigation") {
        line = `[navigation] ${node.name}`;
      } else if (node.role === "main") {
        line = `[main landmark] ${node.name}`;
      } else if (node.role === "img") {
        line = `[image] ${node.name}`;
      } else if (node.role === "form") {
        line = `[form] ${node.name}`;
      } else if (node.name) {
        line = `[${node.role}] ${node.name}`;
      }

      if (line) lines.push(line);
    }

    return lines.join("\n");
  }

  private renderNode(node: A11yNode): string {
    const roleColor = node.missingSemantic ? "#ff4444" : "#ffaa22";
    const icon = node.missingSemantic ? "⚠" : node.focusOrder > 0 ? `⊞` : "○";
    const focusBadge = node.focusOrder > 0
      ? `<span style="color:#4488ff;font-size:10px;margin-left:4px">[tab ${node.focusOrder}]</span>`
      : "";

    return `
      <div style="padding:2px 4px;font-size:11px;border-bottom:1px solid #222">
        <span style="color:#555">${icon}</span>
        <span style="color:${roleColor}">${node.role}</span>
        <span style="color:#ddd">${truncate(node.name, 30)}</span>
        ${focusBadge}
      </div>
    `;
  }
}

function inferRole(el: HTMLElement): string {
  const tag = el.tagName.toLowerCase();
  const roleMap: Record<string, string> = {
    a: "link", button: "button", input: "textbox", select: "combobox",
    textarea: "textbox", img: "img", nav: "navigation", main: "main",
    header: "banner", footer: "contentinfo", form: "form",
    h1: "heading", h2: "heading", h3: "heading", h4: "heading",
  };
  return roleMap[tag] || "generic";
}

function getAccessibleName(el: HTMLElement): string {
  return (
    el.getAttribute("aria-label") ||
    el.getAttribute("alt") ||
    el.getAttribute("title") ||
    el.textContent?.trim().slice(0, 50) ||
    ""
  );
}

function getAriaStates(el: HTMLElement): Record<string, string> {
  const states: Record<string, string> = {};
  for (const attr of el.attributes) {
    if (attr.name.startsWith("aria-") && attr.name !== "aria-label" && attr.name !== "aria-labelledby") {
      states[attr.name] = attr.value;
    }
  }
  return states;
}

function truncate(s: string, max: number): string {
  return s.length > max ? s.slice(0, max) + "…" : s;
}
