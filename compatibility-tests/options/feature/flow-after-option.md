# Flow After Option

Verify execution continues at parent level after option content.

## Script
```cuentitos
Question
  * Choice One
    Inside choice one
      Nested content
  * Choice Two
    Inside choice two
Back at parent level
```

## Input
```input
1
s
```

## Result
```result
START
Question
  1. Choice One
  2. Choice Two
> Selected: Choice One
Inside choice one
Nested content
Back at parent level
END
```
