# Jump Up Multiple Levels

This test verifies that jumping up multiple levels using .. \ .. works correctly.

## Script
```cuentitos
# Root
Text in root
  ## Level 1
  Text in level 1
    ### Level 2
    Text in level 2
    -> .. \ ..

# Another Root
Text in another root
```

## Input
```input
s
```

## Result
```result
START
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
```

## Input
```input
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
n
q
```

## Result
```result
START
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
Text in root
-> Root \ Level 1
Text in level 1
-> Root \ Level 1 \ Level 2
Text in level 2
-> Root
QUIT
```
