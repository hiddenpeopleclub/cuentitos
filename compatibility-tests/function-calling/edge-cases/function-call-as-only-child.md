# Edge Case: Function Call As The Only Child

A function call fires the same way whether it shares its parent with other
lines or stands alone as the sole child. This proves a solitary function
call is not skipped or treated as a special case just because it has no
siblings.

## Script
```cuentitos
You find a strange machine.
  `play_sound alarm`
The machine hums.
```

## Input
```input
s
```

## Result
```result
START
You find a strange machine.
CALL play_sound alarm
The machine hums.
END
```
