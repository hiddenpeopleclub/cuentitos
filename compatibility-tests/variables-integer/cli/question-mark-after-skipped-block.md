# CLI: `?` Output After a Block Is Skipped by a Failing `req`

When a block is skipped by a failing `req`, it does not appear in output and
`?` does not mention it. The state printed by `?` reflects the actual
variable values, independent of which blocks were shown.

## Script
```cuentitos
--- variables
int health = 10
int gold = 0
---

You are alive.
  req health > 0
You are rich.
  req gold > 100
You press on.
```

## Input
```input
n
n
?
s
```

## Result
```result
START
You are alive.
You press on.
health: 10
gold: 0
END
```
