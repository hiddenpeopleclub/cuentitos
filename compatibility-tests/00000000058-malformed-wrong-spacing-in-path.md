# Error: Malformed Wrong Spacing Around Backslash

This test verifies that incorrect spacing around backslash results in a parse error.

## Script
```cuentitos
# Root
  ## Section A
  Text in A

# Section B
-> Root\Section A
```

## Input
```input
s
```

## Result
```result
00000000058-malformed-wrong-spacing-in-path.cuentitos:6: ERROR: Expected section names separated by ' \\ '
```
