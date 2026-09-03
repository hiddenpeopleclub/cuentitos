# Bucket Nested Under a Hidden One-Off Draws Nothing

A bucket can be nested as the child of a probabilistic one-off. When the
parent one-off's own roll hides it, its whole subtree is skipped, including
the nested bucket and the bucket's own draw. A trailing one-off right after
proves this: it receives the very next draw in the sequence, showing that no
draw was spent on the hidden bucket.

## Script
```cuentitos
(50%) A stranger approaches and offers a wager.
  (50%) You win the wager.
  (50%) You lose the wager.
(80%) A second stranger passes by and nods.
```

## Input
```input
seed 1244000820892258425
s
```

## Result
```result
START
A second stranger passes by and nods.
END
```
