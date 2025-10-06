# Tree Traversal Continues After Jumped Section

This test verifies that after executing a jumped section, tree traversal continues normally.

## Script
```cuentitos
# Section A
-> Section C

# Section B
Text in B (skipped)

# Section C
Text in C

# Section D
Text in D (should execute after C)
```

## Input
```input
s
```

## Result
```result
START
-> Section A
-> Section C
Text in C
-> Section D
Text in D (should execute after C)
END
```
