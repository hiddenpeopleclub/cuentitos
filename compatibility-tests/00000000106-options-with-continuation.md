# Options With Continuation

Non-option sibling after options - flow continues at parent level.

## Script
```cuentitos
What do you want?
  * Option A
    You chose A
  * Option B
    You chose B
  The story continues
```

## Input
```input
1
s
```

## Result
```result
START
What do you want?
  1. Option A
  2. Option B
> Selected: Option A
You chose A
The story continues
END
```
