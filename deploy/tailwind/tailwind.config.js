// komadome-rs 用 Tailwind 設定。
//
// content としてはテンプレート(*.ntzr)を走査する。
// 例外:
//   - 動的に決まる要素は safelist で明示
//   - 編集可能なトップページ本文の任意クラスは、生成物の index.html(1ファイル)を追加走査
//     （KOMADOME_TAILWIND_CONTENT=出力先。run-build.sh が BUILD_DIR を渡す）
// forms / typography / aspect-ratio は Tailwind スタンドアロンCLIに同梱されているので require 可。
const defaultTheme = require('tailwindcss/defaultTheme')

const buildDir = process.env.KOMADOME_TAILWIND_CONTENT || '/app/build'

module.exports = {
  content: [
    '/app/templates/**/*.ntzr',
    buildDir + '/index.html',
  ],
  safelist: require('./safelist.json'),
  theme: {
    extend: {
      typography: (theme) => ({
        nonpreflight: {
          css: {
            maxWidth: 'none',
            '--tw-prose-links': theme('colors.blue[600]'),
            '--tw-prose-invert-links': theme('colors.blue[400]'),
            blockquote: {
              fontStyle: 'normal',
            },
          },
        },
      }),
      fontFamily: {
        sans: ['Inter var', ...defaultTheme.fontFamily.sans],
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/aspect-ratio'),
    require('@tailwindcss/typography'),
  ],
}
