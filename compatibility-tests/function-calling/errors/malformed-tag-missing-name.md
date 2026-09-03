# Error: Tag With No Name

A `tag` line must be followed by a name. A bare `tag` with nothing after it
is a parse-time error, mirroring the bare `req` case: the keyword alone is
not a complete statement.

## Script
```cuentitos
You check your journal.
The hidden quest.
  tag
You close the journal.
```

## Input
```input
s
```

## Result
```result
malformed-tag-missing-name.cuentitos:3: ERROR: Malformed tag: no name given after 'tag'.
```
