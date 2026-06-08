# Multiple String Variables In One Block

A `--- variables` block may declare multiple string variables, with and
without defaults, including a reference to an earlier one. Declaration
order is preserved for `?`.

## Script
```cuentitos
--- variables
string a
string b = "bee"
string c = "cee"
string d
string e = b
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
a: ""
b: "bee"
c: "cee"
d: ""
e: "bee"
This is the story.
END
```
