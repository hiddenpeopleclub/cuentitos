# Require: Gating a String-Conditioned Block with Children

When a string `req` fails, the parent block **and every one of its
descendants** are skipped — not just the line the `req` is directly attached
to.

## Script
```cuentitos
--- variables
string quest = "active"
---

You check your journal.
The hidden quest.
  req quest = "secret"
  A note about the ritual.
  The smell of incense.
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
You close the journal.
END
```
