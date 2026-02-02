# Duplicate Section IDs

This test verifies that duplicate section ids under the same parent are rejected.

## Script
```cuentitos
# one: First
Text in first
# one: Second
Text in second
```

## Input
```input
s
```

## Result
```result
00000000006-section-id-duplicate.cuentitos:3: ERROR: Duplicate section name: 'one' already exists at this level under '<root>'. Previously defined at line 1.
```
