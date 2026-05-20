# @voce-ir/sdk

Programmatic TypeScript SDK for Voce IR — IR generation, validation,
and compilation from your own application code.

```bash
npm install @voce-ir/sdk
```

```ts
import { VoceClient } from "@voce-ir/sdk";

const voce = new VoceClient();
const result = await voce.validate(irJson);
if (!result.valid) {
  // structured diagnostics with codes, hints, docs URLs, and
  // JSON-Patch fixes for the 17 auto-fixable codes
  for (const d of result.diagnostics) console.error(d);
}
```

## When to use this

If you're building **a Voce-based application** (a builder, a
generator, a custom workflow) and want to drive the pipeline from
your own code rather than the conversational surfaces
([`@voce-ir/cli-chat`](https://npmjs.com/package/@voce-ir/cli-chat))
or MCP
([`@voce-ir/mcp-server`](https://npmjs.com/package/@voce-ir/mcp-server)).

For most users the conversational surfaces are the right entry
point; this SDK is for everything underneath.

## Full project

See [github.com/marc-pelland/voce-ir](https://github.com/marc-pelland/voce-ir).
Site: [voce-ir.xyz](https://voce-ir.xyz).

## License

Apache-2.0. See [LICENSE](./LICENSE).
