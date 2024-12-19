# CLI Requirements for Compatibility Tests

To effectively run compatibility tests, I aim to establish a clear set of
requirements for the runtime's command line interface (CLI).

In this ADR, I will document these requirements to streamline the implementation
process.

### Submitters

- Fran Tufro

## Change Log

- 2023-12-24 - 🧑‍🎄 - [First Draft of ADR created](https://github.com/hiddenpeopleclub/cuentitos/pull/51).

## Referenced Use Case(s)

To simplify the implementation of a new runtime, it must be clear what the
compatibility test runner expects.

## Context

Creating a robust ecosystem for runtimes requires a comprehensive set of
compatibility tests.

These tests will reside in the `compatibility-tests` folder and will be executed
using the `cuentitos-compat` command.

`cuentitos-compat` will invoke the provided `cuentitos` CLI with the tests and
generate a report.

To enable this functionality, we need to define the expected behavior of the
CLI.

## Proposed Design

The CLI should support a `run` command that accepts a `script` file and an
`input_string` sequence:

```bash
./runtime-cli run [script_file_path] --input [input_string]
```

The script can be either a valid or invalid `cuentitos` script file.

- If the script is invalid, the error must be printed to the standard output.
  This ensures we can test for error compatibility.

- If the script is valid, the input should be executed as if a human was
  interacting with the runtime.

The CLI must adhere to the latest CLI Spec, which is available in the
repository's documentation. (The spec is not included here, as it will evolve
frequently during development.)

## Considerations

Since the CLI will support multiple modes, it makes sense to introduce a `run`
mode that can operate interactively or headlessly.

Additional modes may be defined for other use cases as needed.

## Decision

I will create a new document, `cli-spec.md`, to maintain the latest CLI
specifications. This document will serve as both usage documentation and a
resource for developers implementing new runtimes with their associated CLI.

## Other Related ADRs

- [Compatibility Tests](adr/000001-compatibility-tests) - Defines the process
for building compatibility tests.
