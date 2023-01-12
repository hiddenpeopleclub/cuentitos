# Resources

A user can define resources and their types to use inside conditions.

This is done in the configuration file.

These can be of type `integer`, `float`, and `bool`.

```toml
[resources]
health = "integer"
money = "integer"
```

Once defined, you can use them inside of an event, both as requirements and effects:

```
You find a 
  req money 10
```
