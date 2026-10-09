#!/bin/sh
# the naming rules of the tree (tool/tests/test_namerules.rs and
# test_structure.rs): package names follow their directories, and the tree
# keeps the shape docs/ATLAS.toml describes.
set -e
cd "$(dirname "$0")/../.."
exec cargo test -p tests --test test_namerules --test test_structure "$@"
