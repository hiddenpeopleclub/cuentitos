# Option With Nested Content

Option with multiple levels of nested children.

## Script
```cuentitos
What do you want?
  * Explore the cave
    You enter the cave
      It's dark inside
        You hear a sound
  * Stay outside
    You stay outside
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
  1. Explore the cave
  2. Stay outside
> Selected: Explore the cave
You enter the cave
It's dark inside
You hear a sound
END
```
