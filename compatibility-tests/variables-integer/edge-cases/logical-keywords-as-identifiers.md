# Edge Case: Lowercase `and`/`or`/`not` Are Valid Identifiers

Logical operators are uppercase only. The lowercase forms must remain
usable as variable names. A script that declares variables called `and`,
`or`, and `not`:

- evaluates an arithmetic expression over them with normal `*`/`+`
  precedence (`5 + 3 * 2 = 11`), and
- successfully parses a `req` that mixes the lowercase identifiers with
  the uppercase logical operator `AND` — the parser must disambiguate
  identifier vs. keyword by case.

## Script
```cuentitos
--- variables
int and = 5
int or = 3
int not = 2
int result
---
set result = and + or * not
Identifiers and operators coexist.
  req and > 0 AND or > 0 AND not > 0
```

## Input
```input
n
?
s
```

## Result
```result
START
Identifiers and operators coexist.
and: 5
or: 3
not: 2
result: 11
END
```
