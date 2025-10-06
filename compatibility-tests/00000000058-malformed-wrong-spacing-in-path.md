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
test.cuentitos:6: ERROR: Expected section name after '->'
```
