# Tag On A Block With Its Own Text

A tag attaches to a block without replacing its content. `tag <name>` is
written as an indented line under the block it marks, the same way `req`
attaches a condition to the block above it. When the tagged block renders,
its own text prints first, followed by a `TAG <name>` marker line, proving
the tag reached the runtime API alongside the narrative text rather than in
place of it.

## Script
```cuentitos
You check your journal.
The hidden quest.
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
The hidden quest.
TAG dangerous
You close the journal.
END
```
