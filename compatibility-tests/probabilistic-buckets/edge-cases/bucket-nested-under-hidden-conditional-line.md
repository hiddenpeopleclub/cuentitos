# Bucket Nested Under a Hidden Conditional Line Draws Nothing

A bucket can be nested as the child of a conditional line. When the parent
line's own roll hides it, its whole subtree is skipped, including the nested
bucket and the bucket's own draw.

The trailing `(80%)` line proves the bucket spent no draw. It is the only
child of `You keep walking.`, and a bucket needs at least two branches, so
it is a conditional line in its own right. With this seed the draws are
`0.6389`, `0.1635`, `0.8273`: the first hides the parent, and the second
falls under `0.8` so the coin is seen. Had the hidden bucket drawn, the coin
would have read the third draw instead and stayed hidden.

The `(50%)` line at the top level shares that level with two plain lines, so
no bucket forms there either. The nested bucket is the pair of `(50%)`
children, which are the only children of their parent.

## Script
```cuentitos
You walk into the market.
(50%) A stranger approaches and offers a wager.
  (50%) You win the wager.
  (50%) You lose the wager.
You keep walking.
  (80%) A coin glints in the gutter.
```

## Input
```input
seed 1244000820892258425
s
```

## Result
```result
START
You walk into the market.
You keep walking.
A coin glints in the gutter.
END
```
