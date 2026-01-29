# CLI GoTo with Absolute Path

This test verifies that a user can use absolute paths to jump to nested sections.

## Script
```cuentitos
# Root
  ## Child
    ### Grandchild
    Text in grandchild

# Another
Text in another
```

## Input
```input
n
-> Root \ Child \ Grandchild
s
```

## Result
```result
START
-> Root
-> Root \ Child \ Grandchild
Text in grandchild
-> Another
Text in another
END
```
