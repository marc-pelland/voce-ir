//! State machine validation pass.
//!
//! Checks that StateMachines have valid structure: initial state exists,
//! all states are reachable, no deadlocks, transitions reference valid states.

use std::collections::{HashSet, VecDeque};

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, StateMachine, VoceIr};
use crate::passes::ValidationPass;

pub struct StateMachinePass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "STA001",
        summary: "StateMachine has no states defined",
        hint: "Add at least one entry to the `states` array. Each state needs a \
               unique `state_id`; one of them must be marked `initial: true`.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "STA002",
        summary: "StateMachine has no initial state, or has multiple",
        hint: "Set `initial: true` on exactly one state. If multiple states are \
               flagged initial, pick one to start in and unflag the others.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "STA003",
        summary: "Transition declares an event already handled by another transition",
        hint: "Two transitions handle the same event from the same state. Consolidate \
               the logic or differentiate them with a `guard` condition.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "STA004",
        summary: "Effect references an undefined target node",
        hint: "The Effect's `target_node_id` doesn't match a real node. Update it \
               to point to an existing node, or remove the effect.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "REF004",
        summary: "Transition references a state_id that does not exist",
        hint: "The transition's `to_state` (or `from_state`) doesn't match any state \
               in the StateMachine. Check the name and confirm the state is declared.",
        fix_confidence: None,
    },
];

impl ValidationPass for StateMachinePass {
    fn name(&self) -> &'static str {
        "state-machine"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        if let Some(ref children) = root.children {
            self.find_and_check(children, "/root/children", result);
        }
    }
}

impl StateMachinePass {
    fn find_and_check(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            if child.type_name() == "StateMachine" {
                if let Some(sm) = child.as_type::<StateMachine>() {
                    self.check_state_machine(&sm, &path, result);
                }
            }

            // Recurse
            if let Some(grandchildren) = child.children() {
                self.find_and_check(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }

    fn check_state_machine(&self, sm: &StateMachine, path: &str, result: &mut ValidationResult) {
        let states = match &sm.states {
            Some(s) if !s.is_empty() => s,
            _ => {
                // STA001: Must have at least one state
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "STA001".to_string(),
                    message: "StateMachine has no states".to_string(),
                    node_path: path.to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
                return;
            }
        };

        let transitions = sm.transitions.as_deref().unwrap_or(&[]);
        let state_names: HashSet<&str> = states.iter().filter_map(|s| s.name.as_deref()).collect();

        // STA002: Must have exactly one initial state
        let initial_states: Vec<&str> = states
            .iter()
            .filter(|s| s.initial)
            .filter_map(|s| s.name.as_deref())
            .collect();

        if initial_states.is_empty() {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "STA002".to_string(),
                message: "StateMachine has no initial state (set initial: true on one state)"
                    .to_string(),
                node_path: path.to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
            return;
        }

        if initial_states.len() > 1 {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "STA002".to_string(),
                message: format!(
                    "StateMachine has {} initial states (only one allowed): {}",
                    initial_states.len(),
                    initial_states.join(", ")
                ),
                node_path: path.to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
        }

        // Check transitions reference valid states
        for (j, t) in transitions.iter().enumerate() {
            let t_path = format!("{path}/transitions/{j}");

            if let Some(ref from) = t.from {
                if !state_names.contains(from.as_str()) {
                    // REF004: Transition `from` references non-existent state
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "REF004".to_string(),
                        message: format!(
                            "Transition from state \"{from}\" does not exist in this StateMachine"
                        ),
                        node_path: t_path.clone(),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }

            if let Some(ref to) = t.to {
                if !state_names.contains(to.as_str()) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "REF004".to_string(),
                        message: format!(
                            "Transition to state \"{to}\" does not exist in this StateMachine"
                        ),
                        node_path: t_path,
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }
        }

        // STA003: All states must be reachable from initial
        if let Some(initial) = initial_states.first() {
            let reachable = self.find_reachable(initial, transitions);
            for state in states {
                if let Some(ref name) = state.name {
                    if !reachable.contains(name.as_str()) {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "STA003".to_string(),
                            message: format!(
                                "State \"{name}\" is not reachable from initial state \"{initial}\""
                            ),
                            node_path: path.to_string(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
            }
        }

        // STA004: Duplicate transitions (same from + event)
        let mut seen: HashSet<(String, String)> = HashSet::new();
        for (j, t) in transitions.iter().enumerate() {
            if let (Some(from), Some(event)) = (&t.from, &t.event) {
                let key = (from.clone(), event.clone());
                if !seen.insert(key) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "STA004".to_string(),
                        message: format!(
                            "Duplicate transition: state \"{from}\" + event \"{event}\" appears multiple times"
                        ),
                        node_path: format!("{path}/transitions/{j}"),
                        pass: self.name().to_string(),
                    hint: None,
                    });
                }
            }
        }
    }

    /// BFS from initial state to find all reachable states.
    fn find_reachable<'a>(
        &self,
        initial: &'a str,
        transitions: &'a [crate::ir::Transition],
    ) -> HashSet<&'a str> {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();

        visited.insert(initial);
        queue.push_back(initial);

        while let Some(current) = queue.pop_front() {
            for t in transitions {
                if t.from.as_deref() == Some(current) {
                    if let Some(ref to) = t.to {
                        if visited.insert(to.as_str()) {
                            queue.push_back(to.as_str());
                        }
                    }
                }
            }
        }

        visited
    }
}
