# Tab Indentation Error

Tabs should not be allowed for indentation.

## Script
```cuentitos
First line
	Second line with tab
```

## Input
```input
n
```

## Result
```result
00000000007-tab-indentation-error.cuentitos:2: ERROR: Invalid indentation: found tab indentation.
```
