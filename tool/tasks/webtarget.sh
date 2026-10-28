#!/bin/sh
# the web targets, one name each: trunk reads that target's Trunk.toml and
# index.html and serves in watch mode on its own port, so all of them can
# run at once. extra args pass through to trunk.
#
#   tool/tasks/webtarget.sh [uigallery] [serve|build ...]   the gallery, :8080 (the default)
#   tool/tasks/webtarget.sh urnadbapp [serve|build ...]     the .urna explorer, :8081
#   tool/tasks/webtarget.sh graphs [serve|build ...]        the graphs demo, :8082
#   tool/tasks/webtarget.sh primes [serve|build ...]        the primes demo, :8083
#
# e.g. `tool/tasks/webtarget.sh build --release` for a dist build of the
# gallery, `tool/tasks/webtarget.sh urnadbapp build` for the urna app's.
set -e

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
app="uigallery"
case "${1:-}" in
    uigallery | urnadbapp | graphs | primes) app="$1"; shift ;;
esac
case "$app" in
    uigallery) dir="apps/uigallery/web"; port=8080 ;;
    urnadbapp) dir="apps/urnadbapp/web"; port=8081 ;;
    graphs) dir="demo/graphs"; port=8082 ;;
    primes) dir="demo/primes"; port=8083 ;;
esac
cmd="${1:-serve}"
[ $# -gt 0 ] && shift
cd "$REPO_ROOT"
if [ "$cmd" = serve ]; then
    echo ">> $app: open http://127.0.0.1:$port once the build finishes (the first one takes a few minutes)"
fi
exec trunk --config "$dir" "$cmd" "$@"
