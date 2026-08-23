'use client'

import Link from 'next/link'
import { useState } from 'react'

import { OrganizationScreen } from './screens/OrganizationScreen'
import { PeopleScreen } from './screens/PeopleScreen'
import { CargoScreen } from './screens/CargoScreen'
import { DocumentsScreen } from './screens/DocumentsScreen'
import { PpeReviewScreen } from './screens/PpeReviewScreen'

type AdminView = 'documents' | 'cargo' | 'ppe-review' | 'people' | 'organization'

const NAV: { group: string; items: { id: AdminView; label: string }[] }[] = [
  { group: '화물·문서', items: [{ id: 'documents', label: '화물 문서' }, { id: 'cargo', label: '컨테이너·화물' }, { id: 'ppe-review', label: 'MSDS 보호구 검수' }] },
  { group: '조직', items: [{ id: 'people', label: '직원 관리' }, { id: 'organization', label: '부서·직무' }] },
]

export function AdminWorkspace() {
  const [active, setActive] = useState<AdminView>('people')

  return (
    <div className="admin-shell">
      <aside className="admin-sidebar">
        <div className="admin-wordmark"><span>PS</span><strong>PortSyncAI</strong></div>
        <nav aria-label="관리자 메뉴">
          {NAV.map((group) => (
            <section key={group.group}>
              <p>{group.group}</p>
              {group.items.map((item) => <button className={active === item.id ? 'is-current' : ''} key={item.id} onClick={() => setActive(item.id)} type="button">{item.label}</button>)}
            </section>
          ))}
        </nav>
        <div className="admin-sidebar__account"><span>윤</span><div><strong>윤서진</strong><small>안전 관리자</small></div><Link href="/signin">로그아웃</Link></div>
      </aside>
      <main className="admin-main">
        <div className="admin-topline"><span>2026년 8월 23일 일요일 · 주간조</span><div><button type="button">알림 3</button><span>운영 연결 정상</span></div></div>
        {active === 'documents' && <DocumentsScreen />}
        {active === 'cargo' && <CargoScreen />}
        {active === 'ppe-review' && <PpeReviewScreen />}
        {active === 'people' && <PeopleScreen />}
        {active === 'organization' && <OrganizationScreen />}
      </main>
    </div>
  )
}
