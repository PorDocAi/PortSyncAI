import { devupApi } from '@devup-api/next-plugin'
import { DevupUI } from '@devup-ui/next-plugin'
import type { NextConfig } from 'next'

const isTauriBuild = process.env.TAURI_BUILD === 'true'

const nextConfig: NextConfig = {
  output: isTauriBuild ? 'export' : 'standalone',
  trailingSlash: isTauriBuild,
  experimental: {
    optimizePackageImports: ['@devup-ui/reset-css', '@devup-ui/components'],
  },
  reactCompiler: true,
}

export default DevupUI(devupApi(nextConfig))
