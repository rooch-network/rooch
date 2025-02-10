import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  base: '/',
  server: {
    host: '0.0.0.0',
    port: 3000
  },
  define: {
    'process.env.PACKAGE_ID': `"${process.env.VITE_PACKAGE_ID}"`,
  },
})
