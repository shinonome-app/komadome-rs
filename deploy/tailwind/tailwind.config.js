// komadome-rs 用 Tailwind 設定。
// komadome(Ruby) の config/tailwind.config.js を踏襲しつつ、
// content を「生成済みHTML(build/**/*.html)」に向ける。
// → context 経由で注入される動的クラス(bg-rose-50 等)も取りこぼさない。
// forms / typography / aspect-ratio は Tailwind スタンドアロンCLIに同梱されているので require 可。
const defaultTheme = require('tailwindcss/defaultTheme')

module.exports = {
  content: ['/app/build/**/*.html'],
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
