# Two Lines and End

Two lines of text, with two `next` instructions, should then have an `END`
state.

## Script
```cuentitos
This is a single line
This is another line of text
```

## Input
```input
n
n
n
```

## Result
```result
START
This is a single line
This is another line of text
END
```
