# Multiple Option Groups In Sequence

Two separate option prompts one after another.

## Script
```cuentitos
First question
  * Answer A
    You said A
Second question
  * Answer X
    You said X
  * Answer Y
    You said Y
```

## Input
```input
1
2
s
```

## Result
```result
START
First question
  1. Answer A
> Selected: Answer A
You said A
Second question
  1. Answer X
  2. Answer Y
> Selected: Answer Y
You said Y
END
```
