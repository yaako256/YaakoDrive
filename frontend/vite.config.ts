import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      // SameSite=Strict Cookie を壊さないためにプロキシ経由で /api を転送
      '/api': {
        target: 'http://yaakodrive_backend_dev:9090',
        changeOrigin: true,
      },
    },
  },
})
