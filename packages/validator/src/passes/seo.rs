//! SEO validation pass.
//!
//! Ensures IR produces search-engine-friendly output.

use crate::errors::{CodeMeta, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct SeoPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "SEO001",
        summary: "Page is missing PageMetadata or has no title",
    },
    CodeMeta {
        code: "SEO002",
        summary: "PageMetadata title or description length is outside the recommended range",
    },
    CodeMeta {
        code: "SEO003",
        summary: "Page has zero or multiple h1 headings (recommended: exactly 1)",
    },
    CodeMeta {
        code: "SEO007",
        summary: "OpenGraph data is present but missing og:image",
    },
];

impl ValidationPass for SeoPass {
    fn name(&self) -> &'static str {
        "seo"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        // SEO001: ViewRoot should have metadata with title
        match &root.metadata {
            None => {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "SEO001".to_string(),
                    message: "ViewRoot has no PageMetadata — page will lack title, description, and OG tags".to_string(),
                    node_path: "/root".to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
            Some(meta) => {
                // SEO002: Title should exist and be reasonable length
                if meta.title.as_ref().is_none_or(|t| t.is_empty()) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "SEO001".to_string(),
                        message: "PageMetadata title is empty".to_string(),
                        node_path: "/root/metadata".to_string(),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                } else if let Some(ref title) = meta.title {
                    if title.len() > 60 {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "SEO002".to_string(),
                            message: format!(
                                "PageMetadata title is {} characters (recommended: ≤60)",
                                title.len()
                            ),
                            node_path: "/root/metadata".to_string(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }

                // SEO002b: Description length
                if let Some(ref desc) = meta.description {
                    if desc.len() > 160 {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "SEO002".to_string(),
                            message: format!(
                                "PageMetadata description is {} characters (recommended: ≤160)",
                                desc.len()
                            ),
                            node_path: "/root/metadata".to_string(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }

                // SEO007: OG data completeness
                if let Some(ref og) = meta.open_graph {
                    if og.image.as_ref().is_none_or(|i| i.is_empty()) {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "SEO007".to_string(),
                            message: "OpenGraph data is present but missing og:image".to_string(),
                            node_path: "/root/metadata/open_graph".to_string(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
            }
        }

        // SEO003: Check for exactly one h1
        if let Some(ref children) = root.children {
            let mut h1_count = 0;
            self.count_h1s(children, &mut h1_count);
            if h1_count == 0 {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "SEO003".to_string(),
                    message: "Page has no h1 heading".to_string(),
                    node_path: "/root".to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            } else if h1_count > 1 {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    code: "SEO003".to_string(),
                    message: format!("Page has {h1_count} h1 headings (recommended: exactly 1)"),
                    node_path: "/root".to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
        }
    }
}

impl SeoPass {
    fn count_h1s(&self, children: &[ChildNode], count: &mut usize) {
        for child in children {
            if child.type_name() == "TextNode" {
                if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                    if text.heading_level == Some(1) {
                        *count += 1;
                    }
                }
            }
            if let Some(grandchildren) = child.children() {
                self.count_h1s(&grandchildren, count);
            }
        }
    }
}
