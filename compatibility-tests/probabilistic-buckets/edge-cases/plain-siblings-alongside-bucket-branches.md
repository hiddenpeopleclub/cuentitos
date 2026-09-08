# Plain Siblings Sit Alongside a Bucket

Probabilistic siblings form a bucket between them. Plain lines at the same
level stay outside that bucket and always show, in the position they occupy
in the script. The chosen branch renders where it sits, so the bucket's
output can appear between two plain lines.

The draw is `0.6100`, which lands in the second branch's slice, `[0.50, 1.0)`.

## Script
```cuentitos
I step into the kitchen.
  The kettle is on the stove.
  (50%) A cat is asleep on the counter.
  The window is open.
  (50%) A radio murmurs in the corner.
```

## Input
```input
seed 8474225571399870507
s
```

## Result
```result
START
I step into the kitchen.
The kettle is on the stove.
The window is open.
A radio murmurs in the corner.
END
```
