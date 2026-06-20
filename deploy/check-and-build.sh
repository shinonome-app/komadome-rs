#!/usr/bin/env bash
# 管理画面(shinonome /admin/sysadmin)からの「ビルドのみ」要求を拾って run-build.sh を実行する。
# supercronic から毎分呼ばれる。共有ボリューム /app/control（= ホスト /srv/data/komadome-rs-control、
# shinonome(web) 側にもマウント）経由で request を受け取り、status を書き戻す。
set -euo pipefail

CONTROL_DIR="${KOMADOME_CONTROL_DIR:-/var/run/komadome-rs-control}"
REQ="${CONTROL_DIR}/build.request"
STATUS="${CONTROL_DIR}/build.status"
LOCK="${CONTROL_DIR}/build.lock"
LOG="${CONTROL_DIR}/build.log"

# 要求が無ければ何もしない（毎分呼ばれるので軽量に）
[ -e "${REQ}" ] || exit 0

mkdir -p "${CONTROL_DIR}"

# 二重起動防止: 既にビルド中なら今回はスキップ（要求は残り、次の分に拾われる）
exec 9>"${LOCK}"
flock -n 9 || exit 0

# 要求を消費（同じ要求で何度も走らないよう、実行前に消す）
rm -f "${REQ}"

# 手動ビルドは「プレビュー専用 config」で別出力先(/app/build-preview)に生成し、
# 定時ビルド(/app/build)を上書きしない。転送は常にしない（確認用のため）。
BUILD_DIR=/app/build-preview
printf 'running\t%s\n' "$(date '+%F %T %z')" > "${STATUS}"
if KOMADOME_CONFIG=config/komadome.preview.toml KOMADOME_BUILD_DIR="${BUILD_DIR}" DO_RSYNC=0 \
   /usr/local/bin/run-build.sh > "${LOG}" 2>&1; then
  pages=$(find "${BUILD_DIR}" -name '*.html' 2>/dev/null | wc -l | tr -d ' ')
  printf 'done\t%s\tpages=%s\n' "$(date '+%F %T %z')" "${pages}" > "${STATUS}"
else
  printf 'failed\t%s\n' "$(date '+%F %T %z')" > "${STATUS}"
fi
