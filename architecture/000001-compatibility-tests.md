# Compatibility Tests

### Submitters

- Fran Tufro (Hidden People Club)

## Change Log

- 2023-12-21 \[First Draft of ADR created.\]\(...\)

## Use Case(s)

To ensure a healthy ecosystem around the `cuentitos` programming language, we need to make sure that all the runtimes are compatible with each other.

For that we'll create a set of tests that can be run against a command line implementation of the runtime with a very specific input support.

## Context

We need to make sure that all the runtimes are compatible with each other, so that a script written in `cuentitos` can be run in any runtime.

We need a specific ADR for this to document the work I'll do to create the compatibility tests and the runner.

## Proposed Design

This ADR proposes:

1. A Test Case Format
2. A Test Runner 
3. CLI Requirements

I think that with these three elements, we'll be able to create a set of tests that can be run against any runtime.

### Test Case Format

We'll need to create a format for the test cases that we'll use to test the runtimes.

The approach I'm thinking of is to create a Markdown file with a very specific structure:

````markdown
# Test Name

A description of the test.

## Script
```cuentitos 
// The script to run
```

## Input
```input
// A linebreak-separated list of inputs
```

## Result
```result
// The expected output in the command line
```
````

### Test Case Runner

We'll need a tool that can run the test cases and generate a report.

This tool will be created in the `compat` directory, and it will be called `cuentitos-compat`.

To run all the tests you should be able to do:

```bash
./cuentitos-compat [runtime_cli] [test_cases_directory]
```

In the case you wanted to run just one test:

```bash
./cuentitos-compat [runtime_cli] [test_case_file_path] 
```

### CLI requirements

The Runtime CLI should have the following parameters:

```bash
./runtime-cli [script_file_path] [input_file_path]
```

It should return 0 if the script was executed successfully, and 1 if it failed.

It should also print all the output to the standard output to be captured by the test runner.
