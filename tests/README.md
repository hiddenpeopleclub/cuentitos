# `cuentitos` compatibility tests

Here you can find all the compatibility tests.

The idea is that you can use all these tests along with the provided `compat` tool to test your own `cuentitos` runtime.

This serves me as a way to document how all the features of the environment should work and interact, and to make sure that any bugs that are captured during development are not reintroduced in the future.

## How to run the tests

First of all you need to compile the project as it's explained in the main [README](../README.md).

Then you can run the tests with the following command:

```bash
$ ./cuentitos-compat [runtime] tests
```

This will automatically run all the tests and generate an HTML report in compat-results