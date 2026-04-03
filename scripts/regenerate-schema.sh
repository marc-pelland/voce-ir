#!/usr/bin/env bash
# Regenerate FlatBuffers Rust bindings from schema files.
#
# Usage: ./scripts/regenerate-schema.sh
#
# Requires: flatc (FlatBuffers compiler) v24.x+
#
# Strategy: Concatenate all .fbs domain files into a single compilation
# unit in the correct dependency order. The cross-cutting types
# (ChildUnion, ChildNode, VoceDocument) are defined in voce.fbs and
# appended last. Individual .fbs files remain the source of truth for
# editing.

set -euo pipefail

SCHEMA_DIR="packages/schema/schemas"
GEN_DIR="packages/schema/src/generated"
COMBINED="$SCHEMA_DIR/_combined.fbs"

echo "Regenerating FlatBuffers bindings..."

# Clean previous generated files (keep .gitkeep)
find "$GEN_DIR" -name "*.rs" -delete

# Build combined schema in dependency order.
# Each domain file contributes its types. voce.fbs contributes the
# cross-cutting ChildUnion/ChildNode/VoceDocument at the end.
{
  echo "// AUTO-GENERATED: Combined schema for flatc compilation."
  echo "// Do not edit — edit the individual .fbs files instead."
  echo ""
  echo "namespace voce;"
  echo ""

  # Helper: extract content from an .fbs file, stripping include/namespace lines
  # but keeping ALL table/struct/enum/union definitions and field references.
  extract() {
    grep -v "^include \|^namespace " "$1"
  }

  # 1. types.fbs — foundation, no dependencies
  extract "$SCHEMA_DIR/types.fbs"
  echo ""

  # 2. motion.fbs — ReducedMotion needed by navigation
  extract "$SCHEMA_DIR/motion.fbs"
  echo ""

  # 3. navigation.fbs — depends on types + motion
  extract "$SCHEMA_DIR/navigation.fbs"
  echo ""

  # 4. state.fbs — depends on types
  extract "$SCHEMA_DIR/state.fbs"
  echo ""

  # 5. a11y.fbs — depends on types
  extract "$SCHEMA_DIR/a11y.fbs"
  echo ""

  # 6. theming.fbs — depends on types
  extract "$SCHEMA_DIR/theming.fbs"
  echo ""

  # 7. data.fbs — depends on types + state (DataSource)
  extract "$SCHEMA_DIR/data.fbs"
  echo ""

  # 8. i18n.fbs — depends on types
  extract "$SCHEMA_DIR/i18n.fbs"
  echo ""

  # 9. seo.fbs — depends on types
  extract "$SCHEMA_DIR/seo.fbs"
  echo ""

  # 10. forms.fbs — depends on types
  extract "$SCHEMA_DIR/forms.fbs"
  echo ""

  # 11. layout.fbs — depends on types, a11y, seo, i18n; references ChildNode
  extract "$SCHEMA_DIR/layout.fbs"
  echo ""

  # 12. voce.fbs — cross-cutting types (ChildUnion, ChildNode, VoceDocument)
  extract "$SCHEMA_DIR/voce.fbs"

} > "$COMBINED"

# Compile the combined file
flatc --rust -o "$GEN_DIR" "$COMBINED"

# Clean up
rm -f "$COMBINED"

echo "Generated:"
ls -la "$GEN_DIR"/*.rs
echo "Done. $(wc -l < "$GEN_DIR/_combined_generated.rs") lines."
