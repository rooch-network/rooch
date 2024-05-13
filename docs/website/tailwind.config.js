module.exports = {
  content: [
    './components/**/*.{js,tsx}',
    './pages/**/*.{md,mdx}',
    './theme.config.tsx',
    'node_modules/preline/dist/*.js',
  ],
  theme: {
    extend: {
      backgroundImage: {
        'gradient-blogs':
          'linear-gradient(179.19deg, rgba(255, 255, 255, 0.76) 0.69%, rgba(179, 192, 188, 0.76) 50%, rgba(133, 157, 150, 0.76) 99.31%)',
      },
      boxShadow: {
        custom1: '0px 0px 18.38px 0px #FF914B', // 对应第一个阴影
        custom2: '0px 0px 128.69px 0px #FF914B', // 对应第二个阴影
        custom3: '0px 0px 220.61px 0px #FF914B', // 对应第三个阴影
      },
    },
  },
  plugins: [require('preline/plugin')],
  darkMode: 'class',
}
