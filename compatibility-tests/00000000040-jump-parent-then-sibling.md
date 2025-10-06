# Jump Using Combined Path .. \ Sibling

This test verifies that jumping using a combined path with .. works correctly.

## Script
```cuentitos
# Root
  ## Section A
    ### Deep
    Text in deep
    -> .. \ Section B
  ## Section B
  Text in B
```

## Input
```input
s
```

## Result
```result
START
-> Root
-> Root \ Section A
-> Root \ Section A \ Deep
Text in deep
-> Root \ Section B
Text in B
END
```
