# Function Call With Parameters

A backtick function call may carry an arbitrary number of parameters,
space-separated after the function name. Each parameter is forwarded to the
runtime API in call order as part of the parameter vector. The `CALL`
marker line lists the name followed by every parameter, in order, proving
both parameters made it through and neither was dropped or reordered.

## Script
```cuentitos
You approach the jukebox.
`play_sound alarm 0.3`
The music begins.
```

## Input
```input
s
```

## Result
```result
START
You approach the jukebox.
CALL play_sound alarm 0.3
The music begins.
END
```
