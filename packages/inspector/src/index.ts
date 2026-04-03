/**
 * Voce IR Visual Inspector — debug compiled output without reading code.
 *
 * Injects an overlay onto Voce-compiled HTML that lets you:
 * - Click any element to see its IR node properties
 * - Browse the scene graph tree hierarchy
 * - Inspect state machine current state
 * - View computed styles and ARIA attributes
 *
 * @example
 * ```html
 * <script src="voce-inspector.js"></script>
 * <script>VoceInspector.init();</script>
 * ```
 *
 * Or activate via keyboard: Ctrl+Shift+I
 */

export { VoceInspector } from "./overlay/inspector.js";
export { SceneGraphTree } from "./panel/scene-graph.js";
export { PropertyPanel } from "./panel/properties.js";
export { StateMachineVisualizer, type StateMachineInfo, type TransitionRecord } from "./panel/state-machine.js";
export { AnimationTimeline, type AnimationInfo } from "./panel/animation-timeline.js";
export { DataFlowMonitor, type DataNodeInfo } from "./panel/data-flow.js";
export { A11yTreeViewer, type A11yNode } from "./panel/a11y-tree.js";
export { PerformanceProfiler, type FrameMetrics } from "./panel/performance.js";
export { InlineEditor, type ContentEdit, type IrPatch } from "./editor/inline-editor.js";
export { LocalDraftAdapter, PublishFlow, type CmsBridgeAdapter } from "./editor/cms-bridge.js";
export { DebugAgent, type BugReport, type DebugDiagnosis, type DebugConversation } from "./debug/debug-agent.js";
export { StateReplay, type ReplayStep } from "./debug/state-replay.js";
export type { InspectedNode, InspectorState } from "./overlay/types.js";
