# The Chosen Branch Renders Its Own Children

A bucket branch can carry an indented subtree. When the draw picks that
branch, its text shows and its children follow, in order. The children of the
branches that lost the draw stay out of the transcript along with their
parent.

The draw is `0.1916`, which lands in the first branch's slice, `[0, 0.50)`.

## Script
```cuentitos
I open the front door.
  (50%) The hallway is dark.
    I feel for the light switch.
    The bulb flickers on.
  (50%) The hallway is already lit.
    Someone has been here before me.
```

## Input
```input
seed 6947113883557504045
s
```

## Result
```result
START
I open the front door.
The hallway is dark.
I feel for the light switch.
The bulb flickers on.
END
```
