# Deep Nesting (Three Levels)

This test verifies that deep call nesting works correctly.

## Script
```cuentitos
# Section A
A start
<-> Section B
A end

# Section B
B start
<-> Section C
B end

# Section C
C start
<-> Section D
C end

# Section D
D text
```

## Input
```input
s
```

## Result
```result
START
-> Section A
A start
-> Section B
B start
-> Section C
C start
-> Section D
D text
C end
B end
A end
-> Section B
B start
-> Section C
C start
-> Section D
D text
C end
B end
-> Section C
C start
-> Section D
D text
C end
-> Section D
D text
END
```
