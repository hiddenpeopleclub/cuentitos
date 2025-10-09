# Selected Option Display Format

Verify selected option shows as "Selected: [Option Text]".

## Script
```cuentitos
Pick one
  * First choice
    Content here
  * Second choice
    Other content
```

## Input
```input
1
s
```

## Result
```result
START
Pick one
  1. First choice
  2. Second choice
> Selected: First choice
Content here
END
```
