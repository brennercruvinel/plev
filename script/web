#!/bin/sh
# development loop for the web target: trunk reads web/Trunk.toml and
# web/index.html, serves on 127.0.0.1:8080 in watch mode. extra args pass
# through (e.g. `script/web build --release` for a dist build).
set -e

cmd="${1:-serve}"
[ $# -gt 0 ] && shift
exec trunk --config web "$cmd" "$@"
