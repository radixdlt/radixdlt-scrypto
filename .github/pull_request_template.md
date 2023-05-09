## Summary
```
INSTRUCTIONS:
Add summary - one or two sentences explaining the purpose of this PR.
```

## Details
```
INSTRUCTIONS:
Provide further details about the changes, or how they fit into the roadmap.
You can delete this section if it's not useful.
```

## Testing
```
INSTRUCTIONS:
Further details about the tests you've added or manually performed.
You can delete this section if it's not useful.
```

## Update Recommendations
```
INSTRUCTIONS:
This section is to provide recommendations to consumers of this repo about how they
should handle breaking changes, or integrate new features. The two key stakeholder
groups are dApp Developers and Internal Integrators, and there are separate sections
for each.

In order to allow us to compile aggregated update instructions across PRs, please tag the PR
with 0+ of the relevant labels:
* scrypto-lib - Any change to the scrypto library
* sbor - Any breaking change to SBOR encoding/decoding
* manifest - Any change to manifest display, compilation/decompilation
* transaction - Any change which affects the compiled manifest, or transaction semantics
* substate - Any change to substates, the state model, or what's stored in the DB
* native-blueprint-interface - Any change to the interface of native blueprints
* files-moved - Any change to where engine types have moved, which will require
  internal integrators to update their imports

If you have a breaking change which doesn't fix into a category above, then tag it with
the closest label, and discuss in slack/discord.

If your PR contains no breaking changes or new features or hasn't been tagged, this whole
section can be deleted.
```

### For dApp Developers
```
INSTRUCTIONS:
Migration recommendations for a dApp developer to update their dApp/integrations
due to to the change/s in this PR.

These will be aggregated by the Developer Ecosystem team and included in the next Scrypto migration guide
(eg https://docs-babylon.radixdlt.com/main/scrypto/release_notes/migrating_from_0.7_to_0.8.html )
```

### For Internal Integrators
```
INSTRUCTIONS:
Instructions to any internal integrations (eg Node, Toolkit, Gateway, Ledger App) for how the changes may affect
them and recommendations for how they should update to support this change.
```
