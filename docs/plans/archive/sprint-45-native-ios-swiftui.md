# Sprint 45 — Native: iOS (SwiftUI)

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S38 (v0.4.0 multi-target compilation)

---

## Goal

Build an iOS native compile target that emits SwiftUI code from Voce IR, with VoiceOver accessibility and native gesture mapping.

---

## Deliverables

- `packages/compiler-ios/` Rust crate
- Container → VStack/HStack/ZStack/LazyVGrid mapping
- Surface → SwiftUI shapes with modifiers (cornerRadius, fill, shadow)
- TextNode → Text view with font/color/alignment modifiers
- MediaNode → AsyncImage with placeholder and caching
- StateMachine → @State/@ObservedObject with state enum
- GestureHandler → SwiftUI gesture modifiers (onTapGesture, DragGesture)
- Transition → withAnimation blocks, matchedGeometryEffect
- SemanticNode → accessibility modifiers (accessibilityLabel, accessibilityRole)
- ThemeNode → SwiftUI Environment with ColorScheme
- VoiceOver testing: generated UI navigable with VoiceOver
- `voce compile --target ios` CLI flag
- Output: complete Xcode project or Swift package

---

## Acceptance Criteria

- [ ] Reference landing page IR compiles to working SwiftUI project
- [ ] SwiftUI preview renders correctly in Xcode
- [ ] VoiceOver navigates all interactive elements with correct labels
- [ ] State machines transition correctly on iOS
- [ ] Animations use native SwiftUI animation system
- [ ] Output builds and runs on iOS 17+ simulator
