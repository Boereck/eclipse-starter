[![Build Status](https://travis-ci.org/Boereck/eclipse-starter.svg?branch=master)](https://travis-ci.org/Boereck/eclipse-starter)
![License EPL-2.0](https://img.shields.io/github/license/Boereck/eclipse-starter.svg)

# Eclipse-Starter

NOTE: This project is in an early prototyping phase.

The aim of this project is to rewrite the [Eclipse launcher](https://github.com/eclipse/rt.equinox.framework/tree/master/features/org.eclipse.equinox.executable.feature) executable and library in Rust.
The first step is to rewrite the executable, and still be compatible with the existing library.

The project is structured into three sub-projects:

* `eclipse-launcher` holds the project for the launcher executable, loading the companion library.
* `eclipse-library` is home of the source for the launcher's companion native library.
* `eclipse-common` contains library code shared between `eclipse-launcher` and `eclipse-library`.

# Build

The build is currently very easy. The current version of Rust has to be installed and `cargo build` has to be called in the root directory of the project.
To build a release binary call `cargo build --release`. To build a windows binary that starts a console window, call `cargo build --bin eclipsec --features win_console`.

Just like the original C library build, the following environment variables can be defined to set default os, arch, and ws values:

- `DEFAULT_OS`
- `DEFAULT_OS_ARCH`
- `DEFAULT_WS`

However, these variables do not have to be set. The Rust build will select default values based on the build target.