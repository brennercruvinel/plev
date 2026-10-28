#!/bin/sh
# the web targets, one name each: trunk reads that target's Trunk.toml and
# index.html and serves in watch mode. each target starts from its own port;
# when that one is taken (another target, a server left running), serve takes
# the next free one and says which. extra args pass through to trunk, and a
# --port given here wins.
#
#   tool/tasks/webtarget.sh [uigallery] [serve|build ...]   the gallery, from :8080 (the default)
#   tool/tasks/webtarget.sh urnadbapp [serve|build ...]     the .urna explorer, from :8081
#   tool/tasks/webtarget.sh graphs [serve|build ...]        the graphs demo, from :8082
#   tool/tasks/webtarget.sh primes [serve|build ...]        the primes demo, from :8083
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

# whether something already listens on 127.0.0.1:$1. lsof on macOS and most
# linux; nc where lsof is missing; with neither, the port counts as free and
# trunk says so if it is not.
taken() {
    if command -v lsof >/dev/null 2>&1; then
        lsof -nP -iTCP:"$1" -sTCP:LISTEN >/dev/null 2>&1
    elif command -v nc >/dev/null 2>&1; then
        nc -z 127.0.0.1 "$1" >/dev/null 2>&1
    else
        return 1
    fi
}

if [ "$cmd" = serve ]; then
    case " $* " in
        *" --port "* | *" --port="*) exec trunk --config "$dir" serve "$@" ;;
    esac
    first="$port"
    while taken "$port"; do
        port=$((port + 1))
        if [ "$port" -gt $((first + 50)) ]; then
            echo ">> $app: no free port from $first to $((first + 50))" >&2
            exit 1
        fi
    done
    [ "$port" = "$first" ] || echo ">> $app: port $first is taken, using $port"
    echo ">> $app: open http://127.0.0.1:$port once the build finishes (the first one takes a few minutes)"
    exec trunk --config "$dir" serve --port "$port" "$@"
fi
exec trunk --config "$dir" "$cmd" "$@"
