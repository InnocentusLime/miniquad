# Maintenance

This document outlines how this fork is maintained. For contribution guidelines, please
see `CONTRIBUTING.MD`.

## Lifetime

This fork is in active development. If I decide to stop it, it will be clearly stated
in the `README.md` file.

## API Changes

This fork will not preserve the existing miniquad API. It has several issues, which cannot
be fixed by altering the underlying implementation.

## Updating

The correctness of miniquad is tested with the examples. Each example has instructions in
its code for ensuring that it actually works.

Before merging the changes into `master`, the miniquad build will be tested against all 
examples on Windows.

Before creating a new release, the `master` branch is tested on the following platforms:

* Windows
* Linux (Ubuntu X11)
* MacOS (Both Metal and OpenGL)
* Web Assembly

Important notes:

* More examples may be added. The list is not fixed
* In fact, the list of tests for miniquad might be updated to be beyond the examples
* The amount of tested platforms may be extended in the future

## Versioning

This fork will do releases. The Cargo manifest will be updated accordingly. The versioning
rules follow semver.

## Road to 1.0.0

This fork is aimed to reach version `1.0.0` at some point: stable API and less bugs. This document
will be updated in July with a roadmap to `1.0.0`.