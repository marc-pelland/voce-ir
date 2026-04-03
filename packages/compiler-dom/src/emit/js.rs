//! JavaScript emitter — generates minimal JS for state machines and event handlers.
//!
//! Only emits JS when the IR has interactive elements (StateMachine, GestureHandler).
//! Pages without interactivity get zero JS.

use crate::compiler_ir::{CompiledGestureHandler, CompiledStateMachine, CompilerIr};

/// Generate the JavaScript for all interactive elements.
/// Returns empty string if no interactivity is needed.
pub fn emit_js(ir: &CompilerIr) -> String {
    if ir.state_machines.is_empty() && ir.gesture_handlers.is_empty() && ir.forms.is_empty() {
        return String::new();
    }

    let mut js = String::with_capacity(1024);

    // State machines
    for sm in &ir.state_machines {
        emit_state_machine(&mut js, sm);
    }

    // Gesture handlers (event listeners)
    let needs_domready = !ir.gesture_handlers.is_empty() || !ir.forms.is_empty();
    if needs_domready {
        js.push_str("document.addEventListener('DOMContentLoaded',()=>{\n");
        for gh in &ir.gesture_handlers {
            emit_gesture_handler(&mut js, gh);
        }
        for form in &ir.forms {
            emit_form_validation(&mut js, form);
        }
        js.push_str("});\n");
    }

    js
}

fn emit_state_machine(js: &mut String, sm: &CompiledStateMachine) {
    let var_name = sm.id.replace('-', "_");

    // State variable
    js.push_str(&format!(
        "const {var_name}={{current:'{}'}};",
        sm.initial_state
    ));

    // Transition table
    js.push_str(&format!("const {var_name}_t={{"));
    for state in &sm.states {
        js.push_str(&format!("'{state}':{{"));
        for t in sm.transitions.iter().filter(|t| t.from == *state) {
            js.push_str(&format!("'{}':", t.event));
            js.push_str(&format!("{{to:'{}'", t.to));
            if let Some(ref effect) = t.effect {
                js.push_str(&format!(",fx:'{effect}'"));
            }
            js.push_str("},");
        }
        js.push_str("},");
    }
    js.push_str("};\n");

    // Transition function
    js.push_str(&format!(
        "function {var_name}_send(e){{const t={var_name}_t[{var_name}.current]?.[e];if(!t)return;{var_name}.current=t.to;}}\n"
    ));
}

fn emit_gesture_handler(js: &mut String, gh: &CompiledGestureHandler) {
    let target_id = &gh.target_node_id;
    let event_type = match gh.gesture_type.as_str() {
        "Tap" | "Click" => "click",
        "Hover" => "mouseenter",
        "Focus" => "focus",
        "DoubleTap" => "dblclick",
        _ => "click",
    };

    // Find element by data-voce-id or by DOM structure
    js.push_str(&format!(
        "  const el_{id}=document.querySelector('[data-voce-id=\"{target_id}\"]');\n",
        id = gh.id.replace('-', "_")
    ));

    js.push_str(&format!("  if(el_{id}){{", id = gh.id.replace('-', "_")));

    // Attach event listener
    if let Some(ref trigger_event) = gh.trigger_event {
        if let Some(ref sm_id) = gh.trigger_state_machine {
            let sm_var = sm_id.replace('-', "_");
            js.push_str(&format!(
                "el_{id}.addEventListener('{event_type}',()=>{{{sm_var}_send('{trigger_event}')}});",
                id = gh.id.replace('-', "_")
            ));
        }
    }

    // Keyboard equivalent
    if let Some(ref key) = gh.keyboard_key {
        let id_var = gh.id.replace('-', "_");
        if let Some(ref trigger_event) = gh.trigger_event {
            if let Some(ref sm_id) = gh.trigger_state_machine {
                let sm_var = sm_id.replace('-', "_");
                js.push_str(&format!(
                    "el_{id_var}.addEventListener('keydown',(e)=>{{if(e.key==='{key}'){{{sm_var}_send('{trigger_event}')}}}});"
                ));
            }
        }
    }

    js.push_str("}\n");
}

fn emit_form_validation(js: &mut String, form: &crate::compiler_ir::CompiledForm) {
    let form_id = &form.id;
    let var = form_id.replace('-', "_");

    js.push_str(&format!(
        "  const {var}_form=document.getElementById('{form_id}');\n"
    ));
    js.push_str(&format!(
        "  if({var}_form){{{var}_form.addEventListener('submit',(e)=>{{\n"
    ));
    js.push_str("    let valid=true;\n");

    for field in &form.fields {
        let field_id = format!("{form_id}-{}", field.name);
        let field_var = field.name.replace('-', "_");

        js.push_str(&format!(
            "    const {field_var}=document.getElementById('{field_id}');\n"
        ));
        js.push_str(&format!(
            "    const {field_var}_err=document.getElementById('{field_id}-error');\n"
        ));

        for rule in &field.validations {
            let check = match rule.rule_type.as_str() {
                "Required" => format!("!{field_var}.value.trim()"),
                "Email" => format!("!/^[^\\s@]+@[^\\s@]+\\.[^\\s@]+$/.test({field_var}.value)"),
                "MinLength" => {
                    let min = rule.value.as_deref().unwrap_or("1");
                    format!("{field_var}.value.length<{min}")
                }
                "MaxLength" => {
                    let max = rule.value.as_deref().unwrap_or("255");
                    format!("{field_var}.value.length>{max}")
                }
                "Pattern" => {
                    let pat = rule.value.as_deref().unwrap_or(".*");
                    format!("!/{pat}/.test({field_var}.value)")
                }
                _ => continue,
            };

            let msg = rule.message.replace('\'', "\\'");
            js.push_str(&format!(
                "    if({check}){{{field_var}_err.textContent='{msg}';{field_var}_err.hidden=false;valid=false;}}else{{{field_var}_err.hidden=true;}}\n"
            ));
        }
    }

    js.push_str("    if(!valid){e.preventDefault();}\n");
    js.push_str("  })}\n");
}
