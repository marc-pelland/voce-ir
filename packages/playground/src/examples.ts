/**
 * Built-in IR examples for the playground gallery.
 */

export const EXAMPLES: Record<string, object> = {
  "landing-page": {
    schema_version_major: 1,
    schema_version_minor: 0,
    root: {
      node_id: "root",
      viewport_width: { value: 1440, unit: "Px" },
      children: [
        {
          value_type: "Container",
          value: {
            node_id: "hero",
            direction: "Column",
            alignment: "Center",
            padding: {
              top: { value: 80, unit: "Px" },
              bottom: { value: 80, unit: "Px" },
              left: { value: 24, unit: "Px" },
              right: { value: 24, unit: "Px" },
            },
            children: [
              {
                value_type: "TextNode",
                value: {
                  node_id: "heading",
                  content: "Build UIs with conversation",
                  heading_level: 1,
                  font_size: { value: 48, unit: "Px" },
                  font_weight: "Bold",
                  color: { r: 255, g: 255, b: 255, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "subtitle",
                  content:
                    "Voce IR is an AI-native intermediate representation for user interfaces. Describe what you want, get production-ready output.",
                  font_size: { value: 20, unit: "Px" },
                  color: { r: 161, g: 161, b: 170, a: 255 },
                },
              },
            ],
          },
        },
      ],
    },
    metadata: {
      title: "Voce IR — AI-native UI",
      description: "Build production UIs through conversation.",
      language: "en",
    },
  },

  card: {
    schema_version_major: 1,
    schema_version_minor: 0,
    root: {
      node_id: "root",
      viewport_width: { value: 400, unit: "Px" },
      children: [
        {
          value_type: "Surface",
          value: {
            node_id: "card",
            fill: { r: 26, g: 29, b: 39, a: 255 },
            corner_radius: { top_left: 12, top_right: 12, bottom_left: 12, bottom_right: 12 },
            padding: {
              top: { value: 24, unit: "Px" },
              bottom: { value: 24, unit: "Px" },
              left: { value: 24, unit: "Px" },
              right: { value: 24, unit: "Px" },
            },
            children: [
              {
                value_type: "TextNode",
                value: {
                  node_id: "card-title",
                  content: "Getting Started",
                  heading_level: 2,
                  font_size: { value: 24, unit: "Px" },
                  font_weight: "SemiBold",
                  color: { r: 228, g: 228, b: 231, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "card-body",
                  content: "Voce IR compiles to zero-dependency HTML. No framework, no runtime, just pure output.",
                  font_size: { value: 14, unit: "Px" },
                  color: { r: 161, g: 161, b: 170, a: 255 },
                },
              },
            ],
          },
        },
      ],
    },
    metadata: {
      title: "Card Example",
      description: "A simple card component.",
      language: "en",
    },
  },

  form: {
    schema_version_major: 1,
    schema_version_minor: 0,
    root: {
      node_id: "root",
      viewport_width: { value: 600, unit: "Px" },
      children: [
        {
          value_type: "Container",
          value: {
            node_id: "form-wrapper",
            direction: "Column",
            padding: {
              top: { value: 40, unit: "Px" },
              bottom: { value: 40, unit: "Px" },
              left: { value: 24, unit: "Px" },
              right: { value: 24, unit: "Px" },
            },
            children: [
              {
                value_type: "TextNode",
                value: {
                  node_id: "form-heading",
                  content: "Contact Us",
                  heading_level: 1,
                  font_size: { value: 32, unit: "Px" },
                  font_weight: "Bold",
                  color: { r: 255, g: 255, b: 255, a: 255 },
                },
              },
              {
                value_type: "FormNode",
                value: {
                  node_id: "contact-form",
                  action: { method: "POST", endpoint: "/api/contact" },
                  fields: [
                    {
                      name: "name",
                      field_type: "Text",
                      label: "Your Name",
                      required: true,
                      autocomplete: "Name",
                    },
                    {
                      name: "email",
                      field_type: "Email",
                      label: "Email Address",
                      required: true,
                      autocomplete: "Email",
                      validation_rules: [
                        { rule_type: "Pattern", pattern: "^[^@]+@[^@]+$", message: "Enter a valid email" },
                      ],
                    },
                    {
                      name: "message",
                      field_type: "Textarea",
                      label: "Message",
                      required: true,
                    },
                  ],
                  submit_label: "Send Message",
                  csrf_token: "auto",
                },
              },
            ],
          },
        },
      ],
    },
    metadata: {
      title: "Contact Form",
      description: "A contact form with validation.",
      language: "en",
    },
  },

  nav: {
    schema_version_major: 1,
    schema_version_minor: 0,
    root: {
      node_id: "root",
      viewport_width: { value: 1024, unit: "Px" },
      children: [
        {
          value_type: "Container",
          value: {
            node_id: "navbar",
            direction: "Row",
            alignment: "Center",
            gap: { value: 32, unit: "Px" },
            padding: {
              top: { value: 16, unit: "Px" },
              bottom: { value: 16, unit: "Px" },
              left: { value: 24, unit: "Px" },
              right: { value: 24, unit: "Px" },
            },
            children: [
              {
                value_type: "TextNode",
                value: {
                  node_id: "brand",
                  content: "Voce",
                  font_size: { value: 20, unit: "Px" },
                  font_weight: "Bold",
                  color: { r: 99, g: 102, b: 241, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "nav-docs",
                  content: "Docs",
                  font_size: { value: 14, unit: "Px" },
                  color: { r: 161, g: 161, b: 170, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "nav-playground",
                  content: "Playground",
                  font_size: { value: 14, unit: "Px" },
                  color: { r: 161, g: 161, b: 170, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "nav-github",
                  content: "GitHub",
                  font_size: { value: 14, unit: "Px" },
                  color: { r: 161, g: 161, b: 170, a: 255 },
                },
              },
            ],
          },
        },
      ],
    },
    metadata: {
      title: "Navigation Bar",
      description: "A horizontal navigation bar.",
      language: "en",
    },
  },

  dashboard: {
    schema_version_major: 1,
    schema_version_minor: 0,
    root: {
      node_id: "root",
      viewport_width: { value: 400, unit: "Px" },
      children: [
        {
          value_type: "Surface",
          value: {
            node_id: "widget",
            fill: { r: 26, g: 29, b: 39, a: 255 },
            corner_radius: { top_left: 8, top_right: 8, bottom_left: 8, bottom_right: 8 },
            padding: {
              top: { value: 20, unit: "Px" },
              bottom: { value: 20, unit: "Px" },
              left: { value: 20, unit: "Px" },
              right: { value: 20, unit: "Px" },
            },
            children: [
              {
                value_type: "TextNode",
                value: {
                  node_id: "metric-label",
                  content: "Monthly Active Users",
                  font_size: { value: 13, unit: "Px" },
                  color: { r: 113, g: 113, b: 122, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "metric-value",
                  content: "12,847",
                  font_size: { value: 36, unit: "Px" },
                  font_weight: "Bold",
                  color: { r: 228, g: 228, b: 231, a: 255 },
                },
              },
              {
                value_type: "TextNode",
                value: {
                  node_id: "metric-change",
                  content: "+14.2% from last month",
                  font_size: { value: 13, unit: "Px" },
                  color: { r: 34, g: 197, b: 94, a: 255 },
                },
              },
            ],
          },
        },
      ],
    },
    metadata: {
      title: "Dashboard Widget",
      description: "A metric dashboard card.",
      language: "en",
    },
  },
};
