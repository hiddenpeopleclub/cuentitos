# All-Probabilistic Options Form a Bucket

When every option at an indentation level carries a probability marker, the
options form a bucket: the draw picks exactly one, and only that option
appears in the menu. The chosen option is still presented for selection, the
same as any other option.

## Script
```cuentitos
I reach a fork in the road.
  * (50%) Take the left path.
    I went left.
  * (50%) Take the right path.
    I went right.
```

## Input
```input
seed 6947113883557504045
s
1
s
```

## Result
```result
START
I reach a fork in the road.
  1. Take the left path.
> Selected: Take the left path.
I went left.
END
```
