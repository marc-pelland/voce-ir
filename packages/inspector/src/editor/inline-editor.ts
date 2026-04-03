/**
 * Inline content editor — click any TextNode to edit it directly.
 *
 * Uses contenteditable with a floating toolbar for basic formatting.
 * Generates IR patches for each edit.
 */

export interface ContentEdit {
  /** Node ID of the edited element. */
  nodeId: string;
  /** Previous content. */
  oldValue: string;
  /** New content. */
  newValue: string;
  /** Timestamp. */
  timestamp: number;
}

export interface IrPatch {
  op: "replace";
  path: string;
  value: string;
}

export class InlineEditor {
  private edits: ContentEdit[] = [];
  private undoStack: ContentEdit[] = [];
  private redoStack: ContentEdit[] = [];
  private activeElement: HTMLElement | null = null;
  private toolbar: HTMLElement | null = null;
  private onChange: ((patch: IrPatch) => void) | null = null;

  /** Enable inline editing on all content elements. */
  enable(onChange?: (patch: IrPatch) => void): void {
    this.onChange = onChange || null;

    // Find all editable text elements (with data-voce-id)
    const textElements = document.querySelectorAll<HTMLElement>(
      "h1[data-voce-id], h2[data-voce-id], h3[data-voce-id], " +
      "h4[data-voce-id], h5[data-voce-id], h6[data-voce-id], " +
      "p[data-voce-id], span[data-voce-id]"
    );

    for (const el of textElements) {
      this.makeEditable(el);
    }

    // Also enable on elements inside voce-id containers
    const containers = document.querySelectorAll<HTMLElement>("[data-voce-id]");
    for (const container of containers) {
      const textChildren = container.querySelectorAll<HTMLElement>("h1, h2, h3, h4, h5, h6, p, span");
      for (const el of textChildren) {
        if (!el.hasAttribute("contenteditable")) {
          this.makeEditable(el);
        }
      }
    }

    this.createToolbar();
  }

  /** Disable inline editing. */
  disable(): void {
    document.querySelectorAll("[contenteditable]").forEach((el) => {
      el.removeAttribute("contenteditable");
      (el as HTMLElement).style.outline = "";
    });
    this.toolbar?.remove();
    this.toolbar = null;
  }

  /** Undo the last edit. */
  undo(): boolean {
    const edit = this.undoStack.pop();
    if (!edit) return false;

    // Find the element and revert
    const el = this.findElementByNodeId(edit.nodeId);
    if (el) {
      el.textContent = edit.oldValue;
    }

    this.redoStack.push(edit);
    return true;
  }

  /** Redo a previously undone edit. */
  redo(): boolean {
    const edit = this.redoStack.pop();
    if (!edit) return false;

    const el = this.findElementByNodeId(edit.nodeId);
    if (el) {
      el.textContent = edit.newValue;
    }

    this.undoStack.push(edit);
    return true;
  }

  /** Get all unsaved edits. */
  getEdits(): ContentEdit[] {
    return [...this.edits];
  }

  /** Generate IR patches for all edits. */
  generatePatches(): IrPatch[] {
    return this.edits.map((edit) => ({
      op: "replace" as const,
      path: `/root/children/*[node_id="${edit.nodeId}"]/value/content`,
      value: edit.newValue,
    }));
  }

  /** Clear all edits (after saving). */
  clearEdits(): void {
    this.edits = [];
    this.undoStack = [];
    this.redoStack = [];
  }

  private makeEditable(el: HTMLElement): void {
    el.setAttribute("contenteditable", "true");
    el.style.cursor = "text";

    // Visual indicator on hover
    el.addEventListener("mouseenter", () => {
      if (!this.activeElement) {
        el.style.outline = "1px dashed rgba(68,136,255,0.4)";
      }
    });
    el.addEventListener("mouseleave", () => {
      if (el !== this.activeElement) {
        el.style.outline = "";
      }
    });

    // Track focus
    el.addEventListener("focus", () => {
      this.activeElement = el;
      el.style.outline = "2px solid #4488ff";
      this.showToolbar(el);
    });

    // Track edits on blur
    const originalText = el.textContent || "";
    el.addEventListener("blur", () => {
      const newText = el.textContent || "";
      if (newText !== originalText) {
        const nodeId = this.findNodeId(el);
        if (nodeId) {
          const edit: ContentEdit = {
            nodeId,
            oldValue: originalText,
            newValue: newText,
            timestamp: Date.now(),
          };
          this.edits.push(edit);
          this.undoStack.push(edit);
          this.redoStack = [];

          // Notify
          this.onChange?.({
            op: "replace",
            path: `/root/children/*[node_id="${nodeId}"]/value/content`,
            value: newText,
          });
        }
      }
      this.activeElement = null;
      el.style.outline = "";
      this.hideToolbar();
    });
  }

  private findNodeId(el: HTMLElement): string | null {
    // Check self
    if (el.dataset.voceId) return el.dataset.voceId;
    // Check parent
    const parent = el.closest<HTMLElement>("[data-voce-id]");
    return parent?.dataset.voceId || null;
  }

  private findElementByNodeId(nodeId: string): HTMLElement | null {
    return document.querySelector(`[data-voce-id="${nodeId}"]`);
  }

  private createToolbar(): void {
    this.toolbar = document.createElement("div");
    this.toolbar.id = "voce-edit-toolbar";
    this.toolbar.style.cssText =
      "position:fixed;display:none;background:#2a2a3e;border:1px solid #444;border-radius:6px;" +
      "padding:4px 8px;gap:4px;z-index:100001;box-shadow:0 4px 12px rgba(0,0,0,0.3);";
    this.toolbar.innerHTML = `
      <span style="color:#888;font-size:11px;padding:0 4px">Editing</span>
      <button onclick="document.querySelector('#voce-edit-toolbar').__undoFn?.()" style="background:none;border:none;color:#aaa;cursor:pointer;font-size:12px" title="Undo (Ctrl+Z)">↩</button>
      <button onclick="document.querySelector('#voce-edit-toolbar').__redoFn?.()" style="background:none;border:none;color:#aaa;cursor:pointer;font-size:12px" title="Redo (Ctrl+Y)">↪</button>
    `;
    (this.toolbar as any).__undoFn = () => this.undo();
    (this.toolbar as any).__redoFn = () => this.redo();
    document.body.appendChild(this.toolbar);
  }

  private showToolbar(el: HTMLElement): void {
    if (!this.toolbar) return;
    const rect = el.getBoundingClientRect();
    this.toolbar.style.display = "flex";
    this.toolbar.style.top = `${rect.top - 36}px`;
    this.toolbar.style.left = `${rect.left}px`;
  }

  private hideToolbar(): void {
    if (this.toolbar) {
      this.toolbar.style.display = "none";
    }
  }
}
