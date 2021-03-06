[![Build Status](https://travis-ci.org/Boereck/eclipse-starter.svg?branch=master)](https://travis-ci.org/Boereck/eclipse-starter)
![License EPL-2.0](https://img.shields.io/github/license/Boereck/eclipse-starter.svg)

# Eclipse-Starter

NOTE: This project is in an early stage. The launcher executable is in feature parity with the C version,
the launcher companion library has no features up until now.

The aim of this project is to rewrite the [Eclipse launcher](https://github.com/eclipse/rt.equinox.framework/tree/master/features/org.eclipse.equinox.executable.feature) executable and library in Rust.
The first step is to rewrite the executable, and still be compatible with the existing library.

The project is structured into three sub-projects:

* `eclipse-launcher` holds the project for the launcher executable, loading the companion library.
* `eclipse-library` is home of the source for the launcher's companion native library.
* `eclipse-common` contains library code shared between `eclipse-launcher` and `eclipse-library`.

## Editor Setup

If you want to edit on the project in Eclipse, start the [Eclipse Installer](https://www.eclipse.org/downloads/packages/installer), 
switch to "Advanced Mode" in the hamburger menu (top right corner) and select "Eclipse IDE for Rust Developers". Click next and 
drag [this Link](https://raw.githubusercontent.com/Boereck/eclipse-starter/master/EclipseStarter.setup) into the tree view 
(e.g. on node Github Projects). Select the newly created "Eclipse Starter" entry and click next. Fill in the required fileds,
click "Finish" and let the installer do it's job.

Or simply edit in the web via GitPod:  
[![Open in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/#https://github.com/Boereck/eclipse-starter)

## Build

The build is currently very easy. The current version of Rust has to be installed and `cargo build` has to be called in the root directory of the project.
To build a release binary call `cargo build --release`. To build a windows binary that starts a console window, call `cargo win_console` from the
root of the project. This command is a shortcut alias for calling `cargo build --bin eclipsec --features win_console --release` from within the sub-project `eclipse-launcher`.

Just like the original C library build, the following environment variables can be defined to set default os, arch, and ws values:

- `DEFAULT_OS`
- `DEFAULT_OS_ARCH`
- `DEFAULT_WS`

However, these variables do not have to be set. The Rust build will select default values based on the build target.

Note that the companion shared library name does not match the name of the current C build. Appearently Cargo's ability to
name output dynamic libraries is limited. So the rename must be done in a separate post-build step.

### Windows

To create a Windows launcher with resource information applied, a Windows SDK needs to be installed.
The easiest way is to install the SDK using [chocolatey](https://chocolatey.org/). Call the following line on an admin PowerShell instance:
```powershell
choco install -y windows-sdk-10
```
When the SDK is not installed, the resource info step fails silently. It is possible to run the build with `cargo build -vv`
to see the output of the build step setting the resource info. This way it can be determined "manually" if this step failed.
When run locally it can easily be spotted if the step failed, since the resulting `eclipse.exe` does not have an icon attached to it.

### Linux

Linux builds need an installed dev version of GTK 3. For Debian based distros install the `libgtk-3-dev` package:

```bash
sudo apt install libgtk-3-dev -y
```

Also note that `glib-2.0` is only supported in version `2.42` or higher. This excludes older distros like Ubuntu 14.04 or older.

# License

The code in this repository is EPL-2.0 licensed. However, there is [one file](eclipse-common/src/messagebox/macos/nsalert.rs) licensed under the MIT license.