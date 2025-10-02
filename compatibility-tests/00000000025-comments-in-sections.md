# Comments in Sections

Comments mixed with section content should be ignored.

## Script
```cuentitos
# First Section
// Comment in first section
This is text in the first section
// Another comment
This is more text

# Second Section
// Comment in second section
This is text in the second section
```

## Input
```input
s
```

## Result
```result
START
-> First Section
This is text in the first section
This is more text
-> Second Section
This is text in the second section
END
```
