#!/bin/sh
# which crate may depend on which (tool/tests/test_depsgraph.rs): the
# libraries under rust/ never depend on an app, a demo or the tooling; the
# apps and demos sit on the libraries; nothing depends on a demo.
set -e
cd "$(dirname "$0")/../.."
exec cargo test -p tests --test test_depsgraph "$@"
