# Duplicate Section Names - Different Levels Allowed

This test verifies that sections can have the same name if they are at different levels or under different parents.

## Script
```cuentitos
# Introduction
This is the root introduction

# Chapter One
This is chapter one
  ## Introduction
  This is fine because it's under Chapter One

# Chapter Two
This is chapter two
  ## Introduction
  This is also fine because it's under Chapter Two
    ### Introduction
    This is fine because it's at a deeper level
```

## Input
```input
s
```

## Result
```result
START
# Introduction
This is the root introduction
# Chapter One
This is chapter one
## Introduction
This is fine because it's under Chapter One
# Chapter Two
This is chapter two
## Introduction
This is also fine because it's under Chapter Two
### Introduction
This is fine because it's at a deeper level
END
```
