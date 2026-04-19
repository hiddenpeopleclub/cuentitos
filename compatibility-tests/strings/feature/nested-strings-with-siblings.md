# Nested Strings with Siblings

A script with three levels of nested strings should display all strings in order,
maintaining proper parent-child relationships and sibling order.

## Script
```cuentitos
This is the parent string
  This is the first child
    This is the first grandchild
    This is the second grandchild
  This is the second child
    This is the third grandchild
    This is the fourth grandchild
  This is the third child
```

## Input
```input
s
```

## Result
```result
START
This is the parent string
This is the first child
This is the first grandchild
This is the second grandchild
This is the second child
This is the third grandchild
This is the fourth grandchild
This is the third child
END
``` 