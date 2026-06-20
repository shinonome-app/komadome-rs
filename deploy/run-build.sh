#!/usr/bin/env bash
#
# public site(www) content generateion pupeline
#
#   export(DB→jsonl) -> generate-zip(CSV) -> build(HTML) -> tailwind(tailwind.css) -> rsync (public site, www)
set -euo pipefail
cd /app

# 1回の実行内で日付を固定（深夜跨ぎでも export/build がズレないように）
KOMADOME_BUILD_DATE="$(date +%F)"
export KOMADOME_BUILD_DATE

echo "[run-build] $(date '+%F %T %z') start (build_date=${KOMADOME_BUILD_DATE})"

komadome export
komadome generate-zip
komadome build

# Tailwind CSS: 生成済みHTMLを走査して必要ユーティリティのみ出力
tailwindcss \
  -i /app/tailwind/input.css \
  -c /app/tailwind/tailwind.config.js \
  -o /app/build/assets/tailwind.css \
  --minify

# 公開サーバへ転送。
# 別管理の静的アセット(inter-font.css / Inter フォント / images / css)は
# --delete で消さないよう除外する（tailwind.css は build/assets にあるので同期される）。
#
# 転送は DO_RSYNC=1 のときだけ行う（既定＝転送しない＝安全側）。
# staging はこれを設定せず生成のみ、production で DO_RSYNC=1 にして公開サーバへ転送する。
if [ "${DO_RSYNC:-0}" = "1" ] && [ -n "${RSYNC_SERVER_PATH:-}" ]; then
  rsync -a --delete \
    --exclude='/assets/inter-font.css' \
    --exclude='/assets/Inter-*' \
    --exclude='/images/' \
    --exclude='/css/' \
    -e "ssh -i ${HOME}/.ssh/id_rsync -o StrictHostKeyChecking=no" \
    /app/build/ "${RSYNC_SERVER_PATH}"
  echo "[run-build] rsync done -> ${RSYNC_SERVER_PATH}"
else
  echo "[run-build] 転送しない（生成のみ。DO_RSYNC=${DO_RSYNC:-0}）"
fi

echo "[run-build] $(date '+%F %T %z') done"
