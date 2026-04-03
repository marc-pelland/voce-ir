//! Voce IR Schema — FlatBuffers type definitions and generated bindings.
//!
//! This crate contains the IR schema definitions and the Rust bindings
//! generated from FlatBuffers `.fbs` files. The schema defines every
//! node type in the Voce IR: layout, state, motion, navigation,
//! accessibility, theming, data, forms, SEO, and i18n.
//!
//! # Architecture
//!
//! The schema is the contract between the AI generation layer and the
//! compilation pipeline. Any tool that produces JSON conforming to this
//! schema can generate valid Voce IR.
//!
//! FlatBuffers `.fbs` files live in `schemas/` and are compiled to Rust
//! bindings in `src/generated/` via `flatc`.
//!
//! # Regenerating Bindings
//!
//! ```bash
//! ./scripts/regenerate-schema.sh
//! ```
//!
//! The script combines all `.fbs` files into a single compilation unit
//! to avoid FlatBuffers cross-module codegen issues, then compiles to Rust.

// All domain schemas are combined into a single generated file by the
// regeneration script. Individual .fbs files remain the source of truth
// for editing; the combined file is a build artifact.
#[allow(
    unused_imports,
    dead_code,
    clippy::all,
    mismatched_lifetime_syntaxes,
    non_snake_case,
    non_camel_case_types,
    missing_docs,
    unsafe_op_in_unsafe_fn
)]
mod generated {
    include!("generated/_combined_generated.rs");
}

pub mod errors;

/// All Voce IR types — re-exported for ergonomic access.
///
/// ```ignore
/// use voce_schema::voce::*;
///
/// // Layout
/// let _ = ContainerLayout::Flex;
/// let _ = FontWeight::Bold;
///
/// // State
/// let _ = CacheStrategy::StaleWhileRevalidate;
///
/// // Motion
/// let _ = GestureType::Tap;
/// let _ = ReducedMotionStrategy::Simplify;
///
/// // Navigation
/// let _ = RouteTransitionType::SharedElement;
/// ```
pub use generated::voce;

#[cfg(test)]
mod tests {
    use super::voce::*;
    use flatbuffers::FlatBufferBuilder;

    #[test]
    fn build_and_read_minimal_document() {
        let mut builder = FlatBufferBuilder::new();

        let content = builder.create_string("Hello, Voce");
        let node_id = builder.create_string("heading");
        let text_node = TextNode::create(
            &mut builder,
            &TextNodeArgs {
                node_id: Some(node_id),
                content: Some(content),
                font_weight: FontWeight::Bold,
                heading_level: 1,
                line_height: 1.5,
                opacity: 1.0,
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::TextNode,
                value: Some(text_node.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let lang = builder.create_string("en");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                document_language: Some(lang),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                schema_version_major: 0,
                schema_version_minor: 1,
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).expect("valid FlatBuffer");
        assert_eq!(doc.schema_version_major(), 0);
        assert_eq!(doc.schema_version_minor(), 1);

        let root = doc.root();
        assert_eq!(root.node_id(), "root");
        assert_eq!(root.document_language(), Some("en"));

        let children = root.children().expect("has children");
        assert_eq!(children.len(), 1);

        let child = children.get(0);
        assert_eq!(child.value_type(), ChildUnion::TextNode);

        let text = child.value_as_text_node().expect("is TextNode");
        assert_eq!(text.node_id(), "heading");
        assert_eq!(text.content(), Some("Hello, Voce"));
        assert_eq!(text.font_weight(), FontWeight::Bold);
        assert_eq!(text.heading_level(), 1);
    }

    #[test]
    fn verify_file_identifier() {
        let mut builder = FlatBufferBuilder::new();

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                ..Default::default()
            },
        );
        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        assert!(flatbuffers::buffer_has_identifier(buf, "VOCE", false));
    }

    #[test]
    fn container_with_layout_properties() {
        let mut builder = FlatBufferBuilder::new();

        let node_id = builder.create_string("main");
        let gap = Length::create(
            &mut builder,
            &LengthArgs {
                value: 16.0,
                unit: LengthUnit::Px,
            },
        );

        let container = Container::create(
            &mut builder,
            &ContainerArgs {
                node_id: Some(node_id),
                layout: ContainerLayout::Flex,
                direction: LayoutDirection::Row,
                main_align: Alignment::SpaceBetween,
                cross_align: Alignment::Center,
                gap: Some(gap),
                wrap: true,
                opacity: 1.0,
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::Container,
                value: Some(container.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let children = doc.root().children().unwrap();
        let container = children.get(0).value_as_container().unwrap();

        assert_eq!(container.node_id(), "main");
        assert_eq!(container.layout(), ContainerLayout::Flex);
        assert_eq!(container.direction(), LayoutDirection::Row);
        assert_eq!(container.main_align(), Alignment::SpaceBetween);
        assert_eq!(container.cross_align(), Alignment::Center);
        assert!(container.wrap());

        let gap = container.gap().unwrap();
        assert_eq!(gap.value(), 16.0);
        assert_eq!(gap.unit(), LengthUnit::Px);
    }

    #[test]
    fn state_machine_creation() {
        let mut builder = FlatBufferBuilder::new();

        let idle = builder.create_string("idle");
        let loading = builder.create_string("loading");
        let loaded = builder.create_string("loaded");

        let state_idle = State::create(
            &mut builder,
            &StateArgs {
                name: Some(idle),
                initial: true,
                terminal: false,
            },
        );
        let state_loading = State::create(
            &mut builder,
            &StateArgs {
                name: Some(loading),
                initial: false,
                terminal: false,
            },
        );
        let state_loaded = State::create(
            &mut builder,
            &StateArgs {
                name: Some(loaded),
                initial: false,
                terminal: true,
            },
        );
        let states = builder.create_vector(&[state_idle, state_loading, state_loaded]);

        // Transitions
        let ev_click = builder.create_string("click");
        let ev_resolve = builder.create_string("resolve");
        let from_idle = builder.create_string("idle");
        let to_loading = builder.create_string("loading");
        let from_loading = builder.create_string("loading");
        let to_loaded = builder.create_string("loaded");

        let t1 = Transition::create(
            &mut builder,
            &TransitionArgs {
                event: Some(ev_click),
                from: Some(from_idle),
                to: Some(to_loading),
                ..Default::default()
            },
        );
        let t2 = Transition::create(
            &mut builder,
            &TransitionArgs {
                event: Some(ev_resolve),
                from: Some(from_loading),
                to: Some(to_loaded),
                ..Default::default()
            },
        );
        let transitions = builder.create_vector(&[t1, t2]);

        let sm_id = builder.create_string("fetch-machine");
        let sm_name = builder.create_string("Fetch Data");
        let sm = StateMachine::create(
            &mut builder,
            &StateMachineArgs {
                node_id: Some(sm_id),
                name: Some(sm_name),
                states: Some(states),
                transitions: Some(transitions),
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::StateMachine,
                value: Some(sm.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let sm = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_state_machine()
            .unwrap();

        assert_eq!(sm.node_id(), "fetch-machine");
        assert_eq!(sm.name(), Some("Fetch Data"));
        // states and transitions are required fields — return Vector directly
        assert_eq!(sm.states().len(), 3);
        assert_eq!(sm.transitions().len(), 2);

        let first_state = sm.states().get(0);
        assert_eq!(first_state.name(), "idle");
        assert!(first_state.initial());

        let first_transition = sm.transitions().get(0);
        assert_eq!(first_transition.event(), "click");
        assert_eq!(first_transition.from(), "idle");
        assert_eq!(first_transition.to(), "loading");
    }

    #[test]
    fn animation_transition_with_spring() {
        let mut builder = FlatBufferBuilder::new();

        let prop_str = builder.create_string("transform.translateY");
        let from_str = builder.create_string("20px");
        let to_str = builder.create_string("0px");
        let prop = AnimatedProperty::create(
            &mut builder,
            &AnimatedPropertyArgs {
                property: Some(prop_str),
                from: Some(from_str),
                to: Some(to_str),
            },
        );
        let props = builder.create_vector(&[prop]);

        let dur = Duration::create(&mut builder, &DurationArgs { ms: 300.0 });

        let easing = Easing::create(
            &mut builder,
            &EasingArgs {
                easing_type: EasingType::Spring,
                stiffness: 300.0,
                damping: 25.0,
                mass: 1.0,
                ..Default::default()
            },
        );

        let rm = ReducedMotion::create(
            &mut builder,
            &ReducedMotionArgs {
                strategy: ReducedMotionStrategy::Remove,
                ..Default::default()
            },
        );

        let target = builder.create_string("hero-text");
        let anim_id = builder.create_string("hero-entrance");
        let anim = AnimationTransition::create(
            &mut builder,
            &AnimationTransitionArgs {
                node_id: Some(anim_id),
                target_node_id: Some(target),
                properties: Some(props),
                duration: Some(dur),
                easing: Some(easing),
                reduced_motion: Some(rm),
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::AnimationTransition,
                value: Some(anim.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let anim = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_animation_transition()
            .unwrap();

        assert_eq!(anim.node_id(), "hero-entrance");
        // target_node_id is required — returns &str directly
        assert_eq!(anim.target_node_id(), "hero-text");

        let easing = anim.easing().unwrap();
        assert_eq!(easing.easing_type(), EasingType::Spring);
        assert_eq!(easing.stiffness(), 300.0);
        assert_eq!(easing.damping(), 25.0);

        let rm = anim.reduced_motion().unwrap();
        assert_eq!(rm.strategy(), ReducedMotionStrategy::Remove);

        // properties is required — returns Vector directly
        let props = anim.properties();
        assert_eq!(props.len(), 1);
        assert_eq!(props.get(0).property(), "transform.translateY");
    }

    #[test]
    fn child_union_covers_all_types() {
        // Verify the ChildUnion has entries for ALL Phase 1 node types (27 total)
        // Layout (S02)
        assert_eq!(ChildUnion::Container.0, 1);
        assert_eq!(ChildUnion::Surface.0, 2);
        assert_eq!(ChildUnion::TextNode.0, 3);
        assert_eq!(ChildUnion::MediaNode.0, 4);
        // State (S03)
        assert_eq!(ChildUnion::StateMachine.0, 5);
        assert_eq!(ChildUnion::DataNode.0, 6);
        assert_eq!(ChildUnion::ComputeNode.0, 7);
        assert_eq!(ChildUnion::EffectNode.0, 8);
        assert_eq!(ChildUnion::ContextNode.0, 9);
        // Motion (S03)
        assert_eq!(ChildUnion::AnimationTransition.0, 10);
        assert_eq!(ChildUnion::Sequence.0, 11);
        assert_eq!(ChildUnion::GestureHandler.0, 12);
        assert_eq!(ChildUnion::ScrollBinding.0, 13);
        assert_eq!(ChildUnion::PhysicsBody.0, 14);
        // Navigation (S03)
        assert_eq!(ChildUnion::RouteMap.0, 15);
        // A11y (S04)
        assert_eq!(ChildUnion::SemanticNode.0, 16);
        assert_eq!(ChildUnion::LiveRegion.0, 17);
        assert_eq!(ChildUnion::FocusTrap.0, 18);
        // Theming (S04)
        assert_eq!(ChildUnion::ThemeNode.0, 19);
        assert_eq!(ChildUnion::PersonalizationSlot.0, 20);
        assert_eq!(ChildUnion::ResponsiveRule.0, 21);
        // Data & Backend (S05)
        assert_eq!(ChildUnion::ActionNode.0, 22);
        assert_eq!(ChildUnion::SubscriptionNode.0, 23);
        assert_eq!(ChildUnion::AuthContextNode.0, 24);
        assert_eq!(ChildUnion::ContentSlot.0, 25);
        assert_eq!(ChildUnion::RichTextNode.0, 26);
        // Forms (S05)
        assert_eq!(ChildUnion::FormNode.0, 27);
    }

    #[test]
    fn semantic_node_with_button_role() {
        let mut builder = FlatBufferBuilder::new();

        let node_id = builder.create_string("sem-cta");
        let role = builder.create_string("button");
        let label = builder.create_string("Add to cart");

        let sem = SemanticNode::create(
            &mut builder,
            &SemanticNodeArgs {
                node_id: Some(node_id),
                role: Some(role),
                label: Some(label),
                tab_index: 0,
                aria_required: false,
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::SemanticNode,
                value: Some(sem.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let sem = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_semantic_node()
            .unwrap();

        assert_eq!(sem.node_id(), "sem-cta");
        assert_eq!(sem.role(), "button");
        assert_eq!(sem.label(), Some("Add to cart"));
        assert_eq!(sem.tab_index(), 0);
    }

    #[test]
    fn live_region_assertive() {
        let mut builder = FlatBufferBuilder::new();

        let node_id = builder.create_string("cart-updates");
        let target = builder.create_string("cart-count");
        let desc = builder.create_string("Shopping cart updates");

        let lr = LiveRegion::create(
            &mut builder,
            &LiveRegionArgs {
                node_id: Some(node_id),
                target_node_id: Some(target),
                politeness: LiveRegionPoliteness::Assertive,
                atomic: true,
                role_description: Some(desc),
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::LiveRegion,
                value: Some(lr.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let lr = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_live_region()
            .unwrap();

        assert_eq!(lr.node_id(), "cart-updates");
        assert_eq!(lr.target_node_id(), "cart-count");
        assert_eq!(lr.politeness(), LiveRegionPoliteness::Assertive);
        assert!(lr.atomic());
        assert_eq!(lr.role_description(), Some("Shopping cart updates"));
    }

    #[test]
    fn theme_node_with_color_palette() {
        let mut builder = FlatBufferBuilder::new();

        let node_id = builder.create_string("theme-dark");
        let name = builder.create_string("dark");

        let colors = ColorPalette::create(
            &mut builder,
            &ColorPaletteArgs {
                background: Some(&Color::new(12, 12, 14, 255)),
                foreground: Some(&Color::new(232, 230, 225, 255)),
                primary: Some(&Color::new(232, 89, 60, 255)),
                surface: Some(&Color::new(20, 20, 23, 255)),
                ..Default::default()
            },
        );

        let theme = ThemeNode::create(
            &mut builder,
            &ThemeNodeArgs {
                node_id: Some(node_id),
                name: Some(name),
                colors: Some(colors),
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::ThemeNode,
                value: Some(theme.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                theme: Some(theme),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();

        // Check theme on document
        let theme = doc.theme().unwrap();
        assert_eq!(theme.name(), "dark");

        let colors = theme.colors().unwrap();
        let bg = colors.background().unwrap();
        assert_eq!(bg.r(), 12);
        assert_eq!(bg.g(), 12);
        assert_eq!(bg.b(), 14);
        assert_eq!(bg.a(), 255);

        let primary = colors.primary().unwrap();
        assert_eq!(primary.r(), 232);
        assert_eq!(primary.g(), 89);
        assert_eq!(primary.b(), 60);
    }

    #[test]
    fn action_node_with_optimistic_update() {
        let mut builder = FlatBufferBuilder::new();

        let endpoint = builder.create_string("https://api.example.com/todos");
        let resource = builder.create_string("todos");
        let source = DataSource::create(
            &mut builder,
            &DataSourceArgs {
                endpoint: Some(endpoint),
                resource: Some(resource),
                ..Default::default()
            },
        );

        let target = builder.create_string("todo-list");
        let optimistic = OptimisticConfig::create(
            &mut builder,
            &OptimisticConfigArgs {
                strategy: OptimisticStrategy::MirrorInput,
                target_data_node_id: Some(target),
                ..Default::default()
            },
        );

        let invalidate = builder.create_string("todo-list");
        let invalidates = builder.create_vector(&[invalidate]);

        let node_id = builder.create_string("create-todo");
        let action = ActionNode::create(
            &mut builder,
            &ActionNodeArgs {
                node_id: Some(node_id),
                source: Some(source),
                method: HttpMethod::POST,
                optimistic: Some(optimistic),
                invalidates: Some(invalidates),
                csrf_protected: true,
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::ActionNode,
                value: Some(action.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let action = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_action_node()
            .unwrap();

        assert_eq!(action.node_id(), "create-todo");
        assert_eq!(action.method(), HttpMethod::POST);
        assert!(action.csrf_protected());

        let opt = action.optimistic().unwrap();
        assert_eq!(opt.strategy(), OptimisticStrategy::MirrorInput);
        assert_eq!(opt.target_data_node_id(), Some("todo-list"));
    }

    #[test]
    fn form_node_with_fields_and_validation() {
        let mut builder = FlatBufferBuilder::new();

        // Email field with validation
        let field_name = builder.create_string("email");
        let field_label = builder.create_string("Email address");
        let placeholder = builder.create_string("you@example.com");

        let req_msg = builder.create_string("Email is required");
        let required = ValidationRule::create(
            &mut builder,
            &ValidationRuleArgs {
                rule_type: ValidationType::Required,
                message: Some(req_msg),
                ..Default::default()
            },
        );

        let email_msg = builder.create_string("Must be a valid email");
        let email_rule = ValidationRule::create(
            &mut builder,
            &ValidationRuleArgs {
                rule_type: ValidationType::Email,
                message: Some(email_msg),
                ..Default::default()
            },
        );

        let validations = builder.create_vector(&[required, email_rule]);

        let email_field = FormField::create(
            &mut builder,
            &FormFieldArgs {
                name: Some(field_name),
                field_type: FormFieldType::Email,
                label: Some(field_label),
                placeholder: Some(placeholder),
                validations: Some(validations),
                autocomplete: AutocompleteHint::Email,
                ..Default::default()
            },
        );

        let fields = builder.create_vector(&[email_field]);

        // Submission
        let action_id = builder.create_string("submit-contact");
        let submission = FormSubmission::create(
            &mut builder,
            &FormSubmissionArgs {
                action_node_id: Some(action_id),
                encoding: FormEncoding::Json,
                progressive: true,
                ..Default::default()
            },
        );

        let node_id = builder.create_string("contact-form");
        let form = FormNode::create(
            &mut builder,
            &FormNodeArgs {
                node_id: Some(node_id),
                fields: Some(fields),
                validation_mode: ValidationMode::OnBlurThenChange,
                submission: Some(submission),
                ..Default::default()
            },
        );

        let child = ChildNode::create(
            &mut builder,
            &ChildNodeArgs {
                value_type: ChildUnion::FormNode,
                value: Some(form.as_union_value()),
            },
        );
        let children = builder.create_vector(&[child]);

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                children: Some(children),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let form = doc
            .root()
            .children()
            .unwrap()
            .get(0)
            .value_as_form_node()
            .unwrap();

        assert_eq!(form.node_id(), "contact-form");
        assert_eq!(form.validation_mode(), ValidationMode::OnBlurThenChange);

        let fields = form.fields();
        assert_eq!(fields.len(), 1);

        let email = fields.get(0);
        assert_eq!(email.name(), "email");
        assert_eq!(email.field_type(), FormFieldType::Email);
        assert_eq!(email.label(), "Email address");
        assert_eq!(email.autocomplete(), AutocompleteHint::Email);

        let validations = email.validations().unwrap();
        assert_eq!(validations.len(), 2);
        assert_eq!(validations.get(0).rule_type(), ValidationType::Required);
        assert_eq!(validations.get(1).rule_type(), ValidationType::Email);

        let sub = form.submission();
        assert_eq!(sub.action_node_id(), "submit-contact");
        assert!(sub.progressive());
    }

    #[test]
    fn document_with_i18n_config() {
        let mut builder = FlatBufferBuilder::new();

        let default_locale = builder.create_string("en-US");
        let fr = builder.create_string("fr-FR");
        let ar = builder.create_string("ar-SA");
        let locales = builder.create_vector(&[default_locale, fr, ar]);

        // Re-create default_locale since the string was consumed
        let default_locale2 = builder.create_string("en-US");
        let mode = builder.create_string("static");

        let i18n = I18nConfig::create(
            &mut builder,
            &I18nConfigArgs {
                default_locale: Some(default_locale2),
                supported_locales: Some(locales),
                mode: Some(mode),
            },
        );

        let root_id = builder.create_string("root");
        let root = ViewRoot::create(
            &mut builder,
            &ViewRootArgs {
                node_id: Some(root_id),
                ..Default::default()
            },
        );

        let doc = VoceDocument::create(
            &mut builder,
            &VoceDocumentArgs {
                root: Some(root),
                i18n: Some(i18n),
                ..Default::default()
            },
        );

        builder.finish(doc, Some("VOCE"));
        let buf = builder.finished_data();

        let doc = flatbuffers::root::<VoceDocument>(buf).unwrap();
        let i18n = doc.i18n().unwrap();
        assert_eq!(i18n.default_locale(), "en-US");
        assert_eq!(i18n.supported_locales().len(), 3);
        assert_eq!(i18n.mode(), Some("static"));
    }
}
