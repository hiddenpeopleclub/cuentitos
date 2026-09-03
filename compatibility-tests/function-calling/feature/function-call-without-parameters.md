# Function Call Without Parameters

A line wrapped in backticks with no content beyond the function name calls
that function with an empty parameter list. Cuentitos does not interpret
the call — it forwards the name to the runtime API. The transcript proves
the call fired with a `CALL <name>` marker line, printed where the block
would otherwise render.

## Script
```cuentitos
You approach the jukebox.
`play_sound`
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
CALL play_sound
The music begins.
END
```
