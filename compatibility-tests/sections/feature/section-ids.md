# Section IDs

This test verifies that explicit ids control navigation while display names are used for rendering.

## Script
```cuentitos
# intro: Introduction
Text in intro
-> next

# next: Next Section
Text in next
```

## Input
```input
s
```

## Result
```result
START
-> Introduction
Text in intro
-> Next Section
Text in next
END
```
