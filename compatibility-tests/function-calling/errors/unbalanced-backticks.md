# Error: Unbalanced Backticks

A function call must open and close its backtick on the same line. A line
that opens with a backtick and never closes it is a parse-time error: the
story never starts.

## Script
```cuentitos
This is the story.
`play_sound alarm
```

## Input
```input
s
```

## Result
```result
unbalanced-backticks.cuentitos:2: ERROR: Unterminated function call: missing closing backtick.
```
