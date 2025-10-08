# Option With GoTo

Option content containing a GoTo statement.

## Script
```cuentitos
What do you want?
  * Visit shop
    Going to shop
    -> Shop
  * Continue
    You continue

# Shop
You are at the shop
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
  1. Visit shop
  2. Continue
> Selected: Visit shop
Going to shop
-> Shop
You are at the shop
END
```
