#!/usr/bin/env bash

set -euo pipefail

# 1) rsync 用SSH鍵を RSYNC_PASS_FILE(env, @NL@区切り) から生成
if [ -n "${RSYNC_PASS_FILE:-}" ]; then
  install -d -m 700 "${HOME}/.ssh"
  printf '%s' "${RSYNC_PASS_FILE}" | sed 's/@NL@/\n/g' > "${HOME}/.ssh/id_rsync"
  chmod 600 "${HOME}/.ssh/id_rsync"
fi

# 2) 起動時に1回ビルド（RUN_ON_START=0 で無効化可）
if [ "${RUN_ON_START:-1}" = "1" ]; then
  /usr/local/bin/run-build.sh || echo "[entrypoint] initial build failed (cron で再試行)"
fi

# 3) supercronic で定時実行（crontab 参照）
exec supercronic /app/crontab
