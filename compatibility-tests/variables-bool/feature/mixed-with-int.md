# Bool And Int Variables In The Same Block

Bool and int variables may coexist in the same `--- variables` block. Each
prints with its own per-type formatting.

## Script
```cuentitos
--- variables
int health = 10
bool is_alive = true
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
health: 10
is_alive: true
This is the story.
END
```
