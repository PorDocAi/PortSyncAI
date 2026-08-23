import type { Metadata } from 'next'

import './styles.css'

export const metadata: Metadata = {
  title: 'PortSyncAI 관리자',
  description: '항만 안전작업 운영 관리자 서비스',
}

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="ko">
      <body>{children}</body>
    </html>
  )
}
