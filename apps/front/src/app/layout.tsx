import { ThemeScript } from '@devup-ui/react'
import type { Metadata } from 'next'

import './styles.css'

export const metadata: Metadata = {
  title: 'PortSyncAI 작업자',
  description: '항만 작업 전 안전조건 확인 서비스',
}

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="ko">
      <body><ThemeScript auto />{children}</body>
    </html>
  )
}
