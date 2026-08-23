'use client'

import Link from 'next/link'
import { useState } from 'react'

import { OrganizationScreen } from './screens/OrganizationScreen'
import { PeopleScreen } from './screens/PeopleScreen'

type AdminView = 'people' | 'organization'

const NAV: { group: string; items: { id: AdminView; label: string }[] }[] = [
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
        {active === 'people' && <PeopleScreen />}
        {active === 'organization' && <OrganizationScreen />}
      </main>
    </div>
  )
}
