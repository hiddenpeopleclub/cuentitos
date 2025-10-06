# Jump from Deeply Nested Subsection to Root

This test verifies that jumping from a deeply nested subsection to a root section works correctly.

## Script
```cuentitos
# Root A
  ## Level 1
    ### Level 2
      #### Level 3
      Text in level 3
      -> Root B

# Root B
Text in root B
```

## Input
```input
s
```

## Result
```result
START
-> Root A
-> Root A \ Level 1
-> Root A \ Level 1 \ Level 2
-> Root A \ Level 1 \ Level 2 \ Level 3
Text in level 3
-> Root B
Text in root B
END
```
