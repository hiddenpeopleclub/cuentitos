# Options In Section

Options within a named section.

## Script
```cuentitos
# MainSection
Choose wisely
  * Left path
    You took the left path
  * Right path
    You took the right path
```

## Input
```input
s
1
s
```

## Result
```result
START
-> MainSection
Choose wisely
  1. Left path
  2. Right path
> Selected: Left path
You took the left path
END
```
