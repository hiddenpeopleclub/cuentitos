# One-Off With Indented Children: Hidden

Same script as `indented-children-shown`, but with a seed whose roll lands
at or above the threshold. The one-off and its whole subtree are skipped —
the same way a failing `req` hides both the block it guards and everything
nested under it.

## Script
```cuentitos
You find a quiet bench.
(50%) A solitary figure sits on the bench, reading.
  The figure glances up and nods at you.
  You nod back.
This serene oasis calms you.
```

## Input
```input
seed 18032983948548825955
s
```

## Result
```result
START
You find a quiet bench.
This serene oasis calms you.
END
```
