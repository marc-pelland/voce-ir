# Sprint 46 — Native: Android (Jetpack Compose)

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S45 (iOS/SwiftUI target)

---

## Goal

Build an Android native compile target that emits Jetpack Compose code from Voce IR, with TalkBack accessibility and Material Design token mapping.

---

## Deliverables

- `packages/compiler-android/` Rust crate
- Container → Column/Row/Box/LazyVerticalGrid mapping
- Surface → Compose Surface with shape, elevation, color
- TextNode → Text composable with TextStyle
- MediaNode → AsyncImage (Coil) with placeholder
- StateMachine → remember/mutableStateOf with sealed class states
- GestureHandler → Modifier.clickable, Modifier.pointerInput
- Transition → animate*AsState, AnimatedVisibility
- SemanticNode → Modifier.semantics with contentDescription, role
- ThemeNode → MaterialTheme with dynamic color support
- TalkBack testing: generated UI navigable with TalkBack
- Material Design 3 token mapping from ThemeNode color palette
- `voce compile --target android` CLI flag
- Output: complete Android Studio project or Gradle module

---

## Acceptance Criteria

- [ ] Reference landing page IR compiles to working Compose project
- [ ] Compose preview renders correctly in Android Studio
- [ ] TalkBack navigates all interactive elements with correct descriptions
- [ ] State machines transition correctly on Android
- [ ] Material Design 3 theming applied from ThemeNode
- [ ] Output builds and runs on Android API 26+ emulator
