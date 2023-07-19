module.exports = {
  content: [
    "./components/**/*.{js,tsx}",
    "./pages/**/*.{md,mdx}",
    "./theme.config.tsx",
    "node_modules/preline/dist/*.js",
  ],
  plugins: [require("preline/plugin")],
  darkMode: "class",
};
