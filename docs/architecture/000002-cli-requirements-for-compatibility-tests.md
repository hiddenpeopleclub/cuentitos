# CLI Requirements for Compatibility Tests

Para poder correr los tests de compatibilidad en forma efectiva, quiero definir un conjunto de requerimientos para la CLI.

En este ADR voy a documentar estos requerimientos para facilitar la implementación del CLI

### Submitters

- Fran Tufro (Hidden People Club)

## Change Log

- 2023-12-24 - 🧑‍🎄 - [First Draft of ADR created](https://github.com/hiddenpeopleclub/cuentitos/pull/51).

## Referenced Use Case(s)

To simplify the process of implementing a new runtime it must be clear what the compatibility test runner is expecting.

## Context

As part of creating a healthy ecosystem for runtimes, we need a set of tests for compatibility.

These tests will be created in the `compatibility-tests` folder, and executed using the `cuentitos-compat` command.

`cuentitos-compat` will call the provided cuentitos CLI with the tests and generate a report.

For this to work, I need to define how that CLI behaves for this purpose.

## Proposed Design

First of all, the CLI should support being called with a `run` command that receives a script file and an input file:

```bash
./runtime-cli run [script_file_path] --input [input_file_path]
```

The script can be both a valid, or invalid `cuentitos` script file.

If it's invalid, the error has to be printed in the standard output.

We care about this because we want to be able to test for error compatibility.

If it's valid, the input should be executed as if there was a human typing it.

The CLI needs to implement the latest CLI Spec, that can be found in the docs section of this repository (I'm not including it here because it will change frequently during development).

## Considerations

The CLI will have multiple modes, so it made sense to add a `run` mode that can be interactive or headless.

I'll define other modes that might be required for different uses.

## Decision

I'll create a new document that's called `cli-spec.md` and will keep the latest spec of the CLI. This can be used both to document the usage and support whoever is implementing a new runtime with its associated CLI.

## Other Related ADRs

- [Compatibility Tests](adr/000001-compatibility-tests) - This defines how compatibility tests are built.
