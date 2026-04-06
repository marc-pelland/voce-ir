//! HTML emitter — generates a complete HTML5 document from the compiler IR.
//!
//! Output is a single self-contained HTML file with:
//! - Inline styles (no external CSS)
//! - Security meta tags (CSP, X-Frame-Options)
//! - ARIA attributes from SemanticNode references
//! - No framework runtime

use crate::compiler_ir::{CNode, CompilerIr, NodeId, NodeKind};
use crate::pipeline::CompileOptions;

/// The complete HTML output.
#[derive(Debug)]
pub struct HtmlOutput {
    pub html: String,
}

impl std::fmt::Display for HtmlOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.html)
    }
}

/// Emit a complete HTML5 document from the compiler IR.
pub fn emit(ir: &CompilerIr, options: &CompileOptions) -> HtmlOutput {
    let mut html = String::with_capacity(4096);

    // DOCTYPE and html open
    let lang = ir.meta.language.as_deref().unwrap_or("en");
    let dir = &ir.meta.text_direction;
    html.push_str("<!DOCTYPE html>\n");
    html.push_str(&format!("<html lang=\"{lang}\" dir=\"{dir}\">\n"));

    // Collect above-fold images for preload hints
    let preload_images: Vec<String> = ir
        .nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::Media {
                src, above_fold, ..
            } if *above_fold => Some(src.clone()),
            _ => None,
        })
        .collect();

    // Head
    emit_head(&mut html, ir, &preload_images, options);

    // Collect IDs that gesture handlers target — these need data-voce-id attributes
    let interactive_targets: std::collections::HashSet<String> = ir
        .gesture_handlers
        .iter()
        .map(|gh| gh.target_node_id.clone())
        .collect();

    // Body
    html.push_str("<body>\n");
    let root = &ir.nodes[ir.root.0];
    for &child_id in &root.children {
        emit_node_safe(&mut html, ir, child_id, 1, options, &interactive_targets);
    }

    // Emit JS if interactive
    let js = crate::emit::js::emit_js(ir);
    if !js.is_empty() {
        html.push_str("<script>\n");
        html.push_str(&js);
        html.push_str("</script>\n");
    }

    html.push_str("</body>\n");
    html.push_str("</html>\n");

    HtmlOutput { html }
}

fn emit_head(
    html: &mut String,
    ir: &CompilerIr,
    preload_images: &[String],
    options: &CompileOptions,
) {
    html.push_str("<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");

    // Content Security Policy
    html.push_str("<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' https: data:\">\n");

    // Security headers
    html.push_str("<meta http-equiv=\"X-Content-Type-Options\" content=\"nosniff\">\n");
    html.push_str("<meta http-equiv=\"X-Frame-Options\" content=\"DENY\">\n");
    html.push_str("<meta name=\"referrer\" content=\"strict-origin-when-cross-origin\">\n");

    // Title
    if let Some(ref title) = ir.meta.title {
        html.push_str(&format!("<title>{}</title>\n", escape_html(title)));
    }

    // Description
    if let Some(ref desc) = ir.meta.description {
        html.push_str(&format!(
            "<meta name=\"description\" content=\"{}\">\n",
            escape_attr(desc)
        ));
    }

    // Canonical URL
    if let Some(ref url) = ir.meta.canonical_url {
        html.push_str(&format!("<link rel=\"canonical\" href=\"{url}\">\n"));
    }

    // Open Graph
    if let Some(ref og_title) = ir.meta.og_title {
        html.push_str(&format!(
            "<meta property=\"og:title\" content=\"{}\">\n",
            escape_attr(og_title)
        ));
    }
    if let Some(ref og_desc) = ir.meta.og_description {
        html.push_str(&format!(
            "<meta property=\"og:description\" content=\"{}\">\n",
            escape_attr(og_desc)
        ));
    }
    if let Some(ref og_img) = ir.meta.og_image {
        html.push_str(&format!(
            "<meta property=\"og:image\" content=\"{og_img}\">\n"
        ));
    }

    // Preload hints for above-fold images
    for src in preload_images {
        html.push_str(&format!(
            "<link rel=\"preload\" as=\"image\" href=\"{src}\" fetchpriority=\"high\">\n"
        ));
    }

    // Font preloads and @font-face (unless --skip-fonts)
    if !options.skip_fonts {
        emit_font_css(html, ir, options);
    }

    // Structured data (JSON-LD)
    for jsonld in &ir.meta.structured_data {
        html.push_str("<script type=\"application/ld+json\">\n");
        html.push_str(jsonld);
        html.push_str("\n</script>\n");
    }

    // Styles
    html.push_str("<style>\n");

    // Theme CSS custom properties
    if !ir.meta.theme_vars.is_empty() {
        html.push_str(":root{");
        for (name, value) in &ir.meta.theme_vars {
            html.push_str(&format!("{name}:{value};"));
        }
        html.push_str("}\n");
    }

    html.push_str("*,*::before,*::after{box-sizing:border-box;margin:0;padding:0}\n");
    html.push_str("body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,sans-serif;line-height:1.5}\n");
    html.push_str("img{max-width:100%;height:auto;display:block}\n");

    // Interactive states for links, buttons, and form inputs
    html.push_str("a{transition:opacity .15s}\n");
    html.push_str("a:hover{opacity:.8}\n");
    html.push_str("a:focus-visible{outline:2px solid var(--voce-primary,#6366f1);outline-offset:2px;border-radius:2px}\n");
    html.push_str("input,textarea,select{transition:border-color .15s,box-shadow .15s}\n");
    html.push_str("input:focus,textarea:focus,select:focus{outline:none;border-color:var(--voce-primary,#6366f1);box-shadow:0 0 0 3px rgba(99,102,241,.2)}\n");
    html.push_str("button,[role=\"button\"]{cursor:pointer}\n");
    html.push_str(".voce-btn{cursor:pointer;transition:opacity .15s,transform .1s}\n");
    html.push_str(".voce-btn:hover{opacity:.9}\n");
    html.push_str(".voce-btn:active{transform:scale(.98)}\n");
    html.push_str(".voce-btn:focus-visible{outline:2px solid var(--voce-primary,#6366f1);outline-offset:2px;border-radius:2px}\n");

    // Animation CSS (Tier 1: CSS transitions with compile-time spring easing)
    for anim in &ir.animations {
        let target = &anim.target_node_id;
        let dur = anim.duration_ms;
        for (prop, _from, _to) in &anim.properties {
            let css_prop = prop.replace("transform.", "");
            html.push_str(&format!(
                "[data-voce-id=\"{target}\"]{{transition:{css_prop} {dur}ms {easing};}}\n",
                easing = anim.easing_css
            ));
        }
    }

    // Reduced motion overrides
    let has_animations = !ir.animations.is_empty();
    if has_animations {
        html.push_str("@media(prefers-reduced-motion:reduce){\n");
        for anim in &ir.animations {
            if anim.has_reduced_motion && anim.reduced_motion_strategy == "Remove" {
                let target = &anim.target_node_id;
                html.push_str(&format!(
                    "[data-voce-id=\"{target}\"]{{transition:none!important;}}\n"
                ));
            }
        }
        html.push_str("}\n");
    }

    // Responsive media queries from ResponsiveRule nodes
    for rule in &ir.responsive_rules {
        html.push_str(&format!(
            "@media(max-width:{}px){{\n",
            rule.min_width_px as u32
        ));
        for (target_id, property, value) in &rule.overrides {
            html.push_str(&format!(
                "[data-voce-id=\"{target_id}\"]{{ {property}:{value}; }}\n"
            ));
        }
        html.push_str("}\n");
    }

    html.push_str("</style>\n");

    html.push_str("</head>\n");
}

/// Safely emit a node, catching panics and emitting an error placeholder on failure.
fn emit_node_safe(
    html: &mut String,
    ir: &CompilerIr,
    node_id: NodeId,
    depth: usize,
    options: &CompileOptions,
    interactive_targets: &std::collections::HashSet<String>,
) {
    // Try to emit the node; on panic, emit a visible error placeholder
    let mut node_html = String::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        emit_node(
            &mut node_html,
            ir,
            node_id,
            depth,
            options,
            interactive_targets,
        );
    }));

    match result {
        Ok(()) => html.push_str(&node_html),
        Err(_) => {
            let indent = "  ".repeat(depth);
            let node_id_str = if node_id.0 < ir.nodes.len() {
                ir.nodes[node_id.0].id.as_str()
            } else {
                "unknown"
            };
            html.push_str(&format!(
                "{indent}<div class=\"voce-error\" style=\"border:2px solid #ef4444;padding:8px;margin:4px;color:#ef4444;font-family:monospace;font-size:12px\" role=\"alert\">\
                 Node &quot;{node_id_str}&quot; failed to compile. See build output for details.\
                 </div>\n"
            ));
        }
    }
}

fn emit_node(
    html: &mut String,
    ir: &CompilerIr,
    node_id: NodeId,
    depth: usize,
    options: &CompileOptions,
    interactive_targets: &std::collections::HashSet<String>,
) {
    let node = &ir.nodes[node_id.0];
    let indent = "  ".repeat(depth);

    // Add data-voce-id if this node is a gesture handler target or debug mode is on
    let data_attr = if interactive_targets.contains(&node.id) || options.debug_attrs {
        format!(" data-voce-id=\"{}\"", node.id)
    } else {
        String::new()
    };

    // Build ARIA attributes from SemanticNode reference
    let aria_attrs = node
        .semantic_node_id
        .as_ref()
        .and_then(|sem_id| ir.semantic_map.get(sem_id))
        .map(|sem| {
            let mut attrs = String::new();
            if let Some(ref role) = sem.role {
                attrs.push_str(&format!(" role=\"{role}\""));
            }
            if let Some(ref label) = sem.label {
                attrs.push_str(&format!(" aria-label=\"{}\"", escape_attr(label)));
            }
            if let Some(ref lb) = sem.labelled_by {
                attrs.push_str(&format!(" aria-labelledby=\"{lb}\""));
            }
            if let Some(ref db) = sem.described_by {
                attrs.push_str(&format!(" aria-describedby=\"{db}\""));
            }
            if let Some(ti) = sem.tab_index {
                if ti >= 0 {
                    attrs.push_str(&format!(" tabindex=\"{ti}\""));
                }
            }
            attrs
        })
        .unwrap_or_default();

    match &node.kind {
        NodeKind::ViewRoot { .. } => {
            // ViewRoot children are emitted directly (already handled in body)
            for &child_id in &node.children {
                emit_node_safe(html, ir, child_id, depth, options, interactive_targets);
            }
        }
        NodeKind::Container {
            layout,
            direction,
            main_align,
            cross_align,
            gap,
            wrap,
        } => {
            let mut style = build_style_string(&node.styles);

            // Layout-specific CSS
            let display = match layout.as_str() {
                "Grid" => "grid",
                _ => "flex",
            };
            style.push_str(&format!("display:{display};"));

            if display == "flex" {
                let dir = match direction.as_str() {
                    "Row" => "row",
                    "RowReverse" => "row-reverse",
                    "ColumnReverse" => "column-reverse",
                    _ => "column",
                };
                style.push_str(&format!("flex-direction:{dir};"));

                let justify = alignment_to_css(main_align);
                if !justify.is_empty() {
                    style.push_str(&format!("justify-content:{justify};"));
                }

                let align = alignment_to_css(cross_align);
                if !align.is_empty() {
                    style.push_str(&format!("align-items:{align};"));
                }

                if *wrap {
                    style.push_str("flex-wrap:wrap;");
                }
            }

            if let Some(g) = gap {
                style.push_str(&format!("gap:{g};"));
            }

            // Map SemanticNode role to semantic HTML element
            let semantic_tag = semantic_html_tag(node, ir);

            html.push_str(&format!(
                "{indent}<{semantic_tag} style=\"{style}\"{aria_attrs}{data_attr}>\n"
            ));
            for &child_id in &node.children {
                emit_node_safe(html, ir, child_id, depth + 1, options, interactive_targets);
            }
            html.push_str(&format!("{indent}</{semantic_tag}>\n"));
        }
        NodeKind::Surface {
            decorative,
            href,
            target,
        } => {
            let style = build_style_string(&node.styles);
            let aria = if *decorative {
                " role=\"presentation\" aria-hidden=\"true\""
            } else {
                ""
            };

            if let Some(url) = href {
                // Surface with href wraps content in <a>
                let target_attr = target
                    .as_ref()
                    .map(|t| format!(" target=\"{t}\""))
                    .unwrap_or_default();
                let rel = if target.as_deref() == Some("_blank") {
                    " rel=\"noopener noreferrer\""
                } else {
                    ""
                };
                html.push_str(&format!(
                    "{indent}<a href=\"{url}\"{target_attr}{rel} class=\"voce-btn\" style=\"{style};display:block;text-decoration:none;color:inherit\"{aria_attrs}{data_attr}>\n"
                ));
                for &child_id in &node.children {
                    emit_node_safe(html, ir, child_id, depth + 1, options, interactive_targets);
                }
                html.push_str(&format!("{indent}</a>\n"));
            } else {
                html.push_str(&format!(
                    "{indent}<div style=\"{style}\"{aria}{aria_attrs}{data_attr}>\n"
                ));
                for &child_id in &node.children {
                    emit_node_safe(html, ir, child_id, depth + 1, options, interactive_targets);
                }
                html.push_str(&format!("{indent}</div>\n"));
            }
        }
        NodeKind::Text {
            content,
            tag,
            href,
            target,
            ..
        } => {
            let style = build_style_string(&node.styles);
            let style_attr = if style.is_empty() {
                String::new()
            } else {
                format!(" style=\"{style}\"")
            };

            if let Some(url) = href {
                // TextNode with href emits <a>
                let target_attr = target
                    .as_ref()
                    .map(|t| format!(" target=\"{t}\""))
                    .unwrap_or_default();
                let rel = if target.as_deref() == Some("_blank") {
                    " rel=\"noopener noreferrer\""
                } else {
                    ""
                };
                // If it's a heading with a link, wrap: <h2><a href>...</a></h2>
                if tag.starts_with('h') {
                    html.push_str(&format!(
                        "{indent}<{tag}{style_attr}><a href=\"{url}\"{target_attr}{rel} style=\"color:inherit;text-decoration:none\">{}</{tag}>\n",
                        escape_html(content)
                    ));
                } else {
                    html.push_str(&format!(
                        "{indent}<a href=\"{url}\"{target_attr}{rel}{style_attr}>{}</a>\n",
                        escape_html(content)
                    ));
                }
            } else {
                html.push_str(&format!(
                    "{indent}<{tag}{style_attr}>{}</{tag}>\n",
                    escape_html(content)
                ));
            }
        }
        NodeKind::Media {
            src,
            alt,
            media_type,
            decorative,
            above_fold,
        } => {
            match media_type.as_str() {
                "Video" => {
                    let alt_attr = if alt.is_empty() {
                        String::new()
                    } else {
                        format!(" aria-label=\"{alt}\"")
                    };
                    html.push_str(&format!(
                        "{indent}<video src=\"{src}\" controls preload=\"metadata\"{alt_attr} style=\"max-width:100%\"></video>\n"
                    ));
                }
                "Audio" => {
                    let alt_attr = if alt.is_empty() {
                        String::new()
                    } else {
                        format!(" aria-label=\"{alt}\"")
                    };
                    html.push_str(&format!(
                        "{indent}<audio src=\"{src}\" controls preload=\"metadata\"{alt_attr}></audio>\n"
                    ));
                }
                _ => {
                    // Image (default)
                    let loading = if *above_fold { "eager" } else { "lazy" };
                    let fetchpriority = if *above_fold {
                        " fetchpriority=\"high\""
                    } else {
                        ""
                    };
                    let alt_attr = if *decorative { "" } else { alt.as_str() };

                    let used_pipeline =
                        emit_image_pipeline(html, src, alt_attr, *above_fold, &indent, options);

                    if !used_pipeline {
                        let has_ext = src.contains('.');
                        let is_image = has_ext
                            && !src.ends_with(".svg")
                            && (src.ends_with(".jpg")
                                || src.ends_with(".jpeg")
                                || src.ends_with(".png")
                                || src.ends_with(".webp")
                                || src.ends_with(".avif"));

                        if is_image {
                            let srcset = crate::assets::generate_srcset(
                                src,
                                crate::assets::RESPONSIVE_WIDTHS,
                            );
                            let sizes = crate::assets::default_sizes();
                            html.push_str(&format!("{indent}<picture>\n"));
                            html.push_str(&format!(
                                "{indent}  <img src=\"{src}\" srcset=\"{srcset}\" sizes=\"{sizes}\" alt=\"{alt_attr}\" loading=\"{loading}\"{fetchpriority} decoding=\"async\">\n"
                            ));
                            html.push_str(&format!("{indent}</picture>\n"));
                        } else {
                            html.push_str(&format!(
                                "{indent}<img src=\"{src}\" alt=\"{alt_attr}\" loading=\"{loading}\"{fetchpriority} decoding=\"async\">\n"
                            ));
                        }
                    }
                }
            }
        }
        NodeKind::RichText { blocks } => {
            emit_rich_text(html, blocks, &indent);
        }
        NodeKind::NonVisual { type_name, .. } => {
            // Most non-visual nodes don't emit HTML (SM, GH, animations → JS).
            // FormNode is the exception — it emits <form> HTML.
            if type_name == "FormNode" {
                if let Some(form) = ir.forms.iter().find(|f| f.id == node.id) {
                    emit_form(html, form, &indent);
                }
            }
        }
    }
}

fn emit_rich_text(html: &mut String, blocks: &[crate::compiler_ir::RichTextBlock], indent: &str) {
    let mut in_list: Option<&str> = None;

    for block in blocks {
        // Close previous list if block type changed
        if let Some(list_tag) = in_list {
            if block.block_type != "ListItem" {
                html.push_str(&format!("{indent}</{list_tag}>\n"));
                in_list = None;
            }
        }

        match block.block_type.as_str() {
            "Paragraph" => {
                html.push_str(&format!("{indent}<p>"));
                emit_rich_text_spans(html, &block.children);
                html.push_str("</p>\n");
            }
            "Heading" => {
                let level = block.level.clamp(1, 6);
                html.push_str(&format!("{indent}<h{level}>"));
                emit_rich_text_spans(html, &block.children);
                html.push_str(&format!("</h{level}>\n"));
            }
            "UnorderedList" => {
                if in_list.is_none() {
                    html.push_str(&format!("{indent}<ul>\n"));
                    in_list = Some("ul");
                }
                // Render child blocks as list items
                for row in &block.rows {
                    html.push_str(&format!("{indent}  <li>"));
                    emit_rich_text_spans(html, &row.children);
                    html.push_str("</li>\n");
                }
            }
            "OrderedList" => {
                if in_list.is_none() {
                    html.push_str(&format!("{indent}<ol>\n"));
                    in_list = Some("ol");
                }
                for row in &block.rows {
                    html.push_str(&format!("{indent}  <li>"));
                    emit_rich_text_spans(html, &row.children);
                    html.push_str("</li>\n");
                }
            }
            "ListItem" => {
                // Standalone list item (shouldn't happen often, but handle it)
                html.push_str(&format!("{indent}<li>"));
                emit_rich_text_spans(html, &block.children);
                html.push_str("</li>\n");
            }
            "CodeBlock" => {
                let lang = block
                    .code_language
                    .as_deref()
                    .map(|l| format!(" class=\"language-{l}\""))
                    .unwrap_or_default();
                html.push_str(&format!("{indent}<pre><code{lang}>"));
                // Code blocks: just emit text without marks
                for span in &block.children {
                    html.push_str(&escape_html(&span.text));
                }
                html.push_str("</code></pre>\n");
            }
            "Blockquote" => {
                html.push_str(&format!("{indent}<blockquote>"));
                emit_rich_text_spans(html, &block.children);
                html.push_str("</blockquote>\n");
            }
            "Divider" => {
                html.push_str(&format!("{indent}<hr>\n"));
            }
            "Image" => {
                if let Some(src) = &block.media_src {
                    let alt = block.media_alt.as_deref().unwrap_or("");
                    html.push_str(&format!(
                        "{indent}<img src=\"{src}\" alt=\"{alt}\" loading=\"lazy\" decoding=\"async\">\n"
                    ));
                }
            }
            "Table" => {
                html.push_str(&format!("{indent}<table>\n"));
                for row in &block.rows {
                    html.push_str(&format!("{indent}  <tr>\n"));
                    if row.block_type == "TableRow" {
                        for cell in &row.rows {
                            html.push_str(&format!("{indent}    <td>"));
                            emit_rich_text_spans(html, &cell.children);
                            html.push_str("</td>\n");
                        }
                    }
                    html.push_str(&format!("{indent}  </tr>\n"));
                }
                html.push_str(&format!("{indent}</table>\n"));
            }
            _ => {
                // Unknown block type — emit as paragraph
                html.push_str(&format!("{indent}<p>"));
                emit_rich_text_spans(html, &block.children);
                html.push_str("</p>\n");
            }
        }
    }

    // Close any remaining open list
    if let Some(list_tag) = in_list {
        html.push_str(&format!("{indent}</{list_tag}>\n"));
    }
}

fn emit_rich_text_spans(html: &mut String, spans: &[crate::compiler_ir::RichTextSpan]) {
    for span in spans {
        let mut text = escape_html(&span.text);

        // Apply marks (innermost first)
        for mark in &span.marks {
            text = match mark.as_str() {
                "Bold" => format!("<strong>{text}</strong>"),
                "Italic" => format!("<em>{text}</em>"),
                "Underline" => format!("<u>{text}</u>"),
                "Strikethrough" => format!("<s>{text}</s>"),
                "Code" => format!("<code>{text}</code>"),
                "Subscript" => format!("<sub>{text}</sub>"),
                "Superscript" => format!("<sup>{text}</sup>"),
                _ => text,
            };
        }

        // Wrap in link if present
        if let Some(url) = &span.link_url {
            text = format!("<a href=\"{url}\">{text}</a>");
        }

        html.push_str(&text);
    }
}

fn emit_form(html: &mut String, form: &crate::compiler_ir::CompiledForm, indent: &str) {
    let method = &form.action_method;
    let action = form.action_endpoint.as_deref().unwrap_or("#");

    html.push_str(&format!(
        "{indent}<form id=\"{id}\" method=\"{method}\" action=\"{action}\" novalidate>\n",
        id = form.id
    ));

    let inner = format!("{indent}  ");
    for field in &form.fields {
        let field_id = format!("{}-{}", form.id, field.name);
        let input_type = match field.field_type.as_str() {
            "Email" => "email",
            "Password" => "password",
            "Number" => "number",
            "Tel" => "tel",
            "Url" => "url",
            "Textarea" => "textarea",
            "Hidden" => "hidden",
            "Date" => "date",
            "Time" => "time",
            "Color" => "color",
            "Range" => "range",
            "File" => "file",
            "Checkbox" => "checkbox",
            "Radio" => "radio",
            "Select" => "select",
            _ => "text",
        };

        // Label
        html.push_str(&format!(
            "{inner}<label for=\"{field_id}\">{label}</label>\n",
            label = escape_html(&field.label)
        ));

        // Description (aria-describedby)
        let describedby = if field.description.is_some() {
            format!(" aria-describedby=\"{field_id}-desc\"")
        } else {
            String::new()
        };

        // Required attribute
        let is_required = field.validations.iter().any(|v| v.rule_type == "Required");
        let required_attr = if is_required {
            " required aria-required=\"true\""
        } else {
            ""
        };

        // Autocomplete
        let autocomplete = field
            .autocomplete
            .as_ref()
            .map(|a| {
                let val = match a.as_str() {
                    "Email" => "email",
                    "Name" => "name",
                    "GivenName" => "given-name",
                    "FamilyName" => "family-name",
                    "NewPassword" => "new-password",
                    "CurrentPassword" => "current-password",
                    "Tel" => "tel",
                    _ => "off",
                };
                format!(" autocomplete=\"{val}\"")
            })
            .unwrap_or_default();

        // Placeholder
        let placeholder = field
            .placeholder
            .as_ref()
            .map(|p| format!(" placeholder=\"{}\"", escape_attr(p)))
            .unwrap_or_default();

        // Input, textarea, select, checkbox, or radio
        match input_type {
            "textarea" => {
                html.push_str(&format!(
                    "{inner}<textarea id=\"{field_id}\" name=\"{name}\"{required_attr}{describedby}{placeholder}></textarea>\n",
                    name = field.name
                ));
            }
            "select" => {
                html.push_str(&format!(
                    "{inner}<select id=\"{field_id}\" name=\"{name}\"{required_attr}{describedby}>\n",
                    name = field.name
                ));
                html.push_str(&format!(
                    "{inner}  <option value=\"\" disabled selected>Choose...</option>\n"
                ));
                for opt in &field.options {
                    html.push_str(&format!(
                        "{inner}  <option value=\"{val}\">{label}</option>\n",
                        val = escape_attr(opt),
                        label = escape_html(opt)
                    ));
                }
                html.push_str(&format!("{inner}</select>\n"));
            }
            "checkbox" => {
                // Checkbox: label wraps the input
                html.push_str(&format!(
                    "{inner}<label><input id=\"{field_id}\" name=\"{name}\" type=\"checkbox\"{required_attr}{describedby}> {label}</label>\n",
                    name = field.name,
                    label = escape_html(&field.label)
                ));
            }
            "radio" => {
                // Radio: one input per option
                for (i, opt) in field.options.iter().enumerate() {
                    let opt_id = format!("{field_id}-{i}");
                    html.push_str(&format!(
                        "{inner}<label><input id=\"{opt_id}\" name=\"{name}\" type=\"radio\" value=\"{val}\"{required_attr}> {label}</label>\n",
                        name = field.name,
                        val = escape_attr(opt),
                        label = escape_html(opt)
                    ));
                }
            }
            _ => {
                html.push_str(&format!(
                    "{inner}<input id=\"{field_id}\" name=\"{name}\" type=\"{input_type}\"{required_attr}{autocomplete}{describedby}{placeholder}>\n",
                    name = field.name
                ));
            }
        }

        // Description text
        if let Some(ref desc) = field.description {
            html.push_str(&format!(
                "{inner}<span id=\"{field_id}-desc\">{}</span>\n",
                escape_html(desc)
            ));
        }

        // Error container (populated by JS validation)
        html.push_str(&format!(
            "{inner}<span id=\"{field_id}-error\" role=\"alert\" aria-live=\"polite\" hidden></span>\n"
        ));
    }

    // Submit button
    html.push_str(&format!("{inner}<button type=\"submit\">Submit</button>\n"));

    html.push_str(&format!("{indent}</form>\n"));
}

fn build_style_string(styles: &std::collections::HashMap<String, String>) -> String {
    if styles.is_empty() {
        return String::new();
    }
    let mut s = String::new();
    // Sort for deterministic output
    let mut pairs: Vec<_> = styles.iter().collect();
    pairs.sort_by_key(|(k, _)| k.as_str());
    for (k, v) in pairs {
        s.push_str(&format!("{k}:{v};"));
    }
    s
}

fn alignment_to_css(align: &str) -> &str {
    match align {
        "Center" => "center",
        "End" => "flex-end",
        "Stretch" => "stretch",
        "SpaceBetween" => "space-between",
        "SpaceAround" => "space-around",
        "SpaceEvenly" => "space-evenly",
        "Baseline" => "baseline",
        _ => "",
    }
}

/// Map a node's SemanticNode role to the correct HTML element.
/// Falls back to "div" if no semantic role is attached.
fn semantic_html_tag<'a>(node: &'a CNode, ir: &'a CompilerIr) -> &'a str {
    node.semantic_node_id
        .as_ref()
        .and_then(|sem_id| ir.semantic_map.get(sem_id))
        .and_then(|sem| sem.role.as_deref())
        .map(|role| match role {
            "navigation" => "nav",
            "main" => "main",
            "contentinfo" => "footer",
            "banner" => "header",
            "complementary" => "aside",
            "region" => "section",
            "article" => "article",
            _ => "div",
        })
        .unwrap_or("div")
}

/// Emit font preload links and @font-face CSS from IR font usage analysis.
fn emit_font_css(html: &mut String, ir: &CompilerIr, options: &CompileOptions) {
    use crate::assets::font_pipeline;

    // Collect font families from node styles (font-family CSS property)
    let mut families: std::collections::HashMap<String, font_pipeline::FontFamilyUsage> =
        std::collections::HashMap::new();

    for node in &ir.nodes {
        if let NodeKind::Text { content, .. } = &node.kind {
            // Extract font-family from the node's styles
            let family = node
                .styles
                .get("font-family")
                .map(|f| f.trim_matches('\'').trim_matches('"').to_string())
                .unwrap_or_else(|| "system-ui".to_string());

            let weight: u16 = node
                .styles
                .get("font-weight")
                .and_then(|w| w.parse().ok())
                .unwrap_or(400);

            let entry = families.entry(family).or_default();
            for ch in content.chars() {
                entry.codepoints.insert(ch as u32);
            }
            entry.weights.insert(weight);
            entry.above_fold = true;
        }
    }

    // Skip system-ui and generic families (no custom font needed)
    families.remove("system-ui");
    families.remove("sans-serif");
    families.remove("serif");
    families.remove("monospace");

    if families.is_empty() {
        return;
    }

    // Emit preload links for above-fold fonts
    for (family, usage) in &families {
        if usage.above_fold {
            if let Some(font_data) = options.font_assets.get(family.as_str()) {
                let hash = font_pipeline::font_content_hash(font_data);
                let filename = format!(
                    "{}-subset-{hash}.woff2",
                    family.to_lowercase().replace(' ', "-")
                );
                let url = if options.assets_dir.is_empty() {
                    format!("/fonts/{filename}")
                } else {
                    format!("{}/fonts/{filename}", options.assets_dir)
                };
                html.push_str(&font_pipeline::preload_link(&url));
                html.push('\n');
            }
        }
    }

    // Emit @font-face blocks
    html.push_str("<style>\n");
    for (family, usage) in &families {
        let range = font_pipeline::unicode_range(&usage.codepoints);

        for &weight in &usage.weights {
            let woff2_url = options.font_assets.get(family.as_str()).map(|data| {
                let hash = font_pipeline::font_content_hash(data);
                let filename = format!(
                    "{}-{weight}-{hash}.woff2",
                    family.to_lowercase().replace(' ', "-")
                );
                if options.assets_dir.is_empty() {
                    format!("/fonts/{filename}")
                } else {
                    format!("{}/fonts/{filename}", options.assets_dir)
                }
            });

            html.push_str(&font_pipeline::font_face_css(
                family,
                weight,
                woff2_url.as_deref(),
                &range,
            ));
            html.push('\n');
        }

        // Fallback font-face with metric overrides
        let stack = font_pipeline::fallback_stack(family);
        if let Some(fallback_css) = font_pipeline::fallback_font_face_css(family, &stack) {
            html.push_str(&fallback_css);
            html.push('\n');
        }
    }
    html.push_str("</style>\n");
}

/// Try to use the real image pipeline. Returns true if it handled the image.
#[cfg(feature = "image-pipeline")]
fn emit_image_pipeline(
    html: &mut String,
    src: &str,
    alt: &str,
    above_fold: bool,
    indent: &str,
    options: &CompileOptions,
) -> bool {
    if let Some(source_bytes) = options.image_assets.get(src) {
        let base_name = std::path::Path::new(src)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");
        if let Ok(processed) = crate::assets::image_pipeline::process_image(source_bytes, base_name)
        {
            let picture = crate::assets::image_pipeline::picture_html(
                &processed,
                alt,
                above_fold,
                &options.assets_dir,
            );
            for line in picture.lines() {
                html.push_str(&format!("{indent}{line}\n"));
            }
            return true;
        }
    }
    false
}

/// Stub when image-pipeline feature is disabled.
#[cfg(not(feature = "image-pipeline"))]
fn emit_image_pipeline(
    _html: &mut String,
    _src: &str,
    _alt: &str,
    _above_fold: bool,
    _indent: &str,
    _options: &CompileOptions,
) -> bool {
    false
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
