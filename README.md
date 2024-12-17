--- 
NOTICE: I'm in the process of reimplementing the whole project. This will be extremely unstable from now on until I remove this notice. Download the latest release, or check the version-0.2 for the latest stable version.
---

# `cuentitos`

`cuentitos` is a probabilistic narrative environment designed to make creating interactive stories **frictionless**.

It's designed from the ground up to make writers more productive, simplifying workflows and assisting the creative process as much as possible.

This repository holds the core components of `cuentitos`, its scripting language, compiler and runtimes.

Please, refer to each project's README.md for specific information.


<details>
<summary>Table of Contents</summary>

- [Components](#components)
- [Philosophy](#philosophy)
- [License](#license)

</details>


## Components

- [ADRs](docs/architecture): Every time I make a design decision, I write an ADR. This is the place to look for the rationale behind the design.
- [Compatibility Test Suite](compatibility-tests): This repository contains all the tests that are run against the different runtimes to ensure compatibility.
- [Language](language): 
- [Compiler](compiler):
- [Reference Runtime](runtime): A reference runtime written in Rust.
- [C++ Runtime](runtime-cpp): A C++ runtime that can be embedded in game engines and consoles.
- [Developer Portal](website): The SSR for [https://dev.cuentitos.studio](https://dev.cuentitos.studio)

## Philosophy

`cuentitos` is expected to be a **powerful** and **extremely productive** environment.

This goal enforces a very opinionated design that is the consequence of hitting my head against the wall for years.

## Build

`cuentitos` is developed using Rust. You need a [working rust installation](https://www.rust-lang.org/tools/install).

Once you have `cargo` running, you just run

```bash
$ cargo build
```

Refer to each project for more information.

## License

`cuentitos` is distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT) for details.
