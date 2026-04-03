/**
 * CMS bridge protocol — adapter interface for headless CMS integration.
 *
 * Content editors push changes through the bridge → CMS saves them →
 * IR is regenerated with updated content.
 */

export interface CmsBridgeAdapter {
  /** Adapter name (e.g., "contentful", "sanity", "strapi"). */
  name: string;

  /** Fetch content for a given content key. */
  fetchContent(contentKey: string): Promise<string>;

  /** Update content in the CMS. */
  updateContent(contentKey: string, value: string): Promise<void>;

  /** Publish all pending changes. */
  publish(): Promise<void>;

  /** Check if there are unsaved changes. */
  hasChanges(): boolean;

  /** Discard all pending changes. */
  discard(): void;
}

/**
 * Local draft adapter — stores edits in-memory for preview.
 * Does not persist to any CMS. Used when no CMS is configured.
 */
export class LocalDraftAdapter implements CmsBridgeAdapter {
  name = "local-draft";
  private drafts = new Map<string, string>();
  private originals = new Map<string, string>();

  async fetchContent(contentKey: string): Promise<string> {
    return this.drafts.get(contentKey) || "";
  }

  async updateContent(contentKey: string, value: string): Promise<void> {
    if (!this.originals.has(contentKey)) {
      this.originals.set(contentKey, this.drafts.get(contentKey) || "");
    }
    this.drafts.set(contentKey, value);
  }

  async publish(): Promise<void> {
    // In local mode, "publish" just clears the originals (commits drafts)
    this.originals.clear();
  }

  hasChanges(): boolean {
    return this.originals.size > 0;
  }

  discard(): void {
    // Revert to originals
    for (const [key, original] of this.originals) {
      this.drafts.set(key, original);
    }
    this.originals.clear();
  }
}

/**
 * Preview/publish flow controller.
 *
 * 1. Content editor makes changes via InlineEditor
 * 2. Changes are sent to CmsBridgeAdapter.updateContent()
 * 3. Preview shows the changes with visual diff indicators
 * 4. Editor clicks "Publish" → adapter.publish() → IR recompiles
 */
export class PublishFlow {
  private adapter: CmsBridgeAdapter;
  private container: HTMLElement | null = null;

  constructor(adapter: CmsBridgeAdapter) {
    this.adapter = adapter;
  }

  /** Render the publish bar at the bottom of the page. */
  renderPublishBar(container: HTMLElement): void {
    this.container = container;
    this.refresh();
  }

  /** Refresh the publish bar state. */
  refresh(): void {
    if (!this.container) return;

    const hasChanges = this.adapter.hasChanges();

    this.container.innerHTML = hasChanges
      ? `<div style="position:fixed;bottom:0;left:0;right:0;background:#2a2a3e;border-top:2px solid #4488ff;padding:12px 24px;display:flex;justify-content:space-between;align-items:center;z-index:100002;font-family:system-ui;font-size:13px">
          <span style="color:#aaa">Unsaved content changes</span>
          <div style="display:flex;gap:8px">
            <button onclick="this.closest('div').parentElement.__discardFn?.()" style="background:transparent;color:#ff8888;border:1px solid #ff4444;border-radius:4px;padding:6px 16px;cursor:pointer">Discard</button>
            <button onclick="this.closest('div').parentElement.__publishFn?.()" style="background:#4488ff;color:#fff;border:none;border-radius:4px;padding:6px 16px;cursor:pointer;font-weight:bold">Publish</button>
          </div>
        </div>`
      : "";

    if (hasChanges) {
      (this.container as any).__discardFn = () => {
        this.adapter.discard();
        this.refresh();
        location.reload();
      };
      (this.container as any).__publishFn = async () => {
        await this.adapter.publish();
        this.refresh();
      };
    }
  }
}
