import type { UserConfig } from 'vite'
import { resolve } from 'path'

export default {
  root: 'vite',
  publicDir: '../test/',
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        ol: resolve(__dirname, 'ol.html'),
        maplibre: resolve(__dirname, 'maplibre.html'),
      },
    },
  }
} satisfies UserConfig