# Nested Sections

This test verifies that sections can be nested with proper indentation and hierarchy.

## Script
```cuentitos
# Main Section
This is text in the main section
  ## First Sub-section
  This is text in the first sub-section
    ### Deep Sub-section
    This is text in a deep sub-section
  ## Second Sub-section
  This is text in the second sub-section

# Another Main Section
This is text in another main section
  ## Its Sub-section
  This is text in its sub-section
```

## Input
```input
n
n
n
n
n
n
n
n
```

## Result
```result
START
-> Main Section
This is text in the main section
-> Main Section \ First Sub-section
This is text in the first sub-section
-> Main Section \ First Sub-section \ Deep Sub-section
This is text in a deep sub-section
-> Main Section \ First Sub-section \ Second Sub-section
This is text in the second sub-section
-> Another Main Section
This is text in another main section
-> Another Main Section \ Its Sub-section
This is text in its sub-section
END
```
