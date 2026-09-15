# Edge Case: A Tag Surfaces Only When Its Block's Req Passes

A tag nested alongside a `req` is gated the same way as any other
descendant: it surfaces when the `req` passes and leaves no trace at all
when the `req` fails, along with the rest of its block. Two quests with the
same tag but opposite `req` outcomes prove the contrast in one script — the
passing quest's block prints its text followed by the `TAG` marker, and the
failing quest disappears entirely, block and tag alike.

## Script
```cuentitos
--- variables
string quest = "secret"
---

You check your journal.
The known quest.
  req quest = "secret"
  tag dangerous
The unknown quest.
  req quest = "mundane"
  tag dangerous
You close the journal.
```

## Input
```input
s
```

## Result
```result
START
You check your journal.
The known quest.
TAG dangerous
You close the journal.
END
```
