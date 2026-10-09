#!/bin/sh
# development loop for the web targets: trunk reads the app's apps/uigallery/web/Trunk.toml
# and apps/uigallery/web/index.html and serves in watch mode, the gallery on 127.0.0.1:8080
# and the urna database app on 127.0.0.1:8081. extra args pass through.
#
#   tool/tasks/webtarget.sh [uigallery] [serve|build ...]   (default: uigallery serve)
#   tool/tasks/webtarget.sh urnadbapp [serve|build ...]
#
# e.g. `tool/tasks/webtarget.sh build --release` for a dist build of the
# gallery, `tool/tasks/webtarget.sh urnadbapp build` for the urna app's.
set -e

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
app="uigallery"
case "${1:-}" in
    uigallery | urnadbapp) app="$1"; shift ;;
esac
cmd="${1:-serve}"
[ $# -gt 0 ] && shift
cd "$REPO_ROOT"
exec trunk --config "apps/$app/web" "$cmd" "$@"
