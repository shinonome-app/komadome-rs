#!/usr/bin/env bash
#
# public site(www) content generateion pupeline
#
#   export(DB→jsonl) -> generate-zip(CSV) -> build(HTML) -> tailwind(tailwind.css) -> rsync (public site, www)
set -euo pipefail
cd /app

# 定時ビルドと手動ビルドで出力先を分けるため、config と出力ディレクトリを切り替え可能にする。
#   - 定時(cron)     : 既定（config/komadome.toml, /app/build）
#   - 手動(sysadmin) : check-and-build.sh が KOMADOME_CONFIG=config/komadome.preview.toml,
#                      KOMADOME_BUILD_DIR=/app/build-preview を渡す
# ※ BUILD_DIR は CONFIG の [output] directory と必ず一致させること（tailwind/rsync が参照）。
CONFIG="${KOMADOME_CONFIG:-config/komadome.toml}"
BUILD_DIR="${KOMADOME_BUILD_DIR:-/app/build}"

# 1回の実行内で日付を固定（深夜跨ぎでも export/build がズレないように）
KOMADOME_BUILD_DATE="$(date +%F)"
export KOMADOME_BUILD_DATE

echo "[run-build] $(date '+%F %T %z') start (config=${CONFIG} out=${BUILD_DIR} build_date=${KOMADOME_BUILD_DATE})"

komadome --config "${CONFIG}" export
komadome --config "${CONFIG}" generate-zip
komadome --config "${CONFIG}" build

# 動的クラス(bgcolor 等)の safelist を Rust(単一の出所)から生成して tailwind に渡す。
komadome tailwind-safelist > /app/tailwind/safelist.json

# Tailwind CSS: テンプレート(*.ntzr)＋編集トップページ(index.html)＋safelist を走査して
# 必要ユーティリティのみ出力（生成物全走査による OOM を避ける）。
KOMADOME_TAILWIND_CONTENT="${BUILD_DIR}" tailwindcss \
  -i /app/tailwind/input.css \
  -c /app/tailwind/tailwind.config.js \
  -o "${BUILD_DIR}/assets/tailwind.css" \
  --minify

# 公開サーバへ転送。
# 追加・更新のみ（--delete はしない＝公開サーバ上の既存ファイルは消さない）。
# 不要になったファイルの削除が必要なときは手動で行う。
# 別管理の静的アセット(inter-font.css / Inter フォント / images / css)は
# komadome-rs の生成物に含まれない＝転送対象にならないため、そのまま残る。
#
# 転送は DO_RSYNC=1 のときだけ行う（既定＝転送しない＝安全側）。
# staging はこれを設定せず生成のみ、production で DO_RSYNC=1 にして公開サーバへ転送する。
if [ "${DO_RSYNC:-0}" = "1" ] && [ -n "${RSYNC_SERVER_PATH:-}" ]; then
  rsync -a \
    -e "ssh -i ${HOME}/.ssh/id_rsync -o StrictHostKeyChecking=no" \
    "${BUILD_DIR}/" "${RSYNC_SERVER_PATH}"
  echo "[run-build] rsync done -> ${RSYNC_SERVER_PATH}"
else
  echo "[run-build] 転送しない（生成のみ。DO_RSYNC=${DO_RSYNC:-0}）"
fi

echo "[run-build] $(date '+%F %T %z') done"
