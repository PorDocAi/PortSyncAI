'use client'

import { Button as UiButton, Grid } from '@devup-ui/react'

import Link from 'next/link'
import { useState } from 'react'

import { OrganizationScreen } from './screens/OrganizationScreen'
import { PeopleScreen } from './screens/PeopleScreen'
import { CargoScreen } from './screens/CargoScreen'
import { DocumentsScreen } from './screens/DocumentsScreen'
import { PpeReviewScreen } from './screens/PpeReviewScreen'
import { AssignmentsScreen } from './screens/AssignmentsScreen'
import { GateHistoryScreen } from './screens/GateHistoryScreen'
import { OperationsScreen } from './screens/OperationsScreen'
import { SafetyScreen } from './screens/SafetyScreen'
import { TrainingScreen } from './screens/TrainingScreen'
import { BrandLockup } from '../BrandLockup'

type AdminView = 'operations' | 'assignments' | 'training' | 'safety' | 'gate-history' | 'documents' | 'cargo' | 'ppe-review' | 'people' | 'organization'

const NAV: { group: string; items: { id: AdminView; label: string }[] }[] = [
  { group: '운영', items: [{ id: 'operations', label: '운영 보드' }, { id: 'assignments', label: '작업·팀 배정' }] },
  { group: '화물·문서', items: [{ id: 'documents', label: '화물 문서' }, { id: 'cargo', label: '컨테이너·화물' }, { id: 'ppe-review', label: 'MSDS 보호구 검수' }] },
  { group: '안전', items: [{ id: 'training', label: '안전교육' }, { id: 'safety', label: '지침·작업중지' }, { id: 'gate-history', label: '게이트 이력' }] },
  { group: '조직', items: [{ id: 'people', label: '직원 관리' }, { id: 'organization', label: '부서·직무' }] },
]

export function AdminWorkspace() {
  const [active, setActive] = useState<AdminView>('operations')

  return (
    <Grid className="admin-shell">
      <aside className="admin-sidebar">
        <BrandLockup />
        <nav aria-label="관리자 메뉴">
          {NAV.map((group) => (
            <section key={group.group}>
              <p>{group.group}</p>
              {group.items.map((item) => <UiButton className={active === item.id ? 'is-current' : ''} key={item.id} onClick={() => setActive(item.id)} type="button">{item.label}</UiButton>)}
            </section>
          ))}
        </nav>
        <div className="admin-sidebar__account"><span>윤</span><div><strong>윤서진</strong><small>안전 관리자</small></div><Link href="/gate-terminal">게이트 단말</Link></div>
      </aside>
      <main className="admin-main">
        <div className="admin-topline"><span>2026년 8월 23일 일요일 · 주간조</span><div><UiButton type="button">알림 3</UiButton><span>운영 연결 정상</span></div></div>
        {active === 'operations' && <OperationsScreen />}
        {active === 'assignments' && <AssignmentsScreen />}
        {active === 'documents' && <DocumentsScreen />}
        {active === 'cargo' && <CargoScreen />}
        {active === 'ppe-review' && <PpeReviewScreen />}
        {active === 'people' && <PeopleScreen />}
        {active === 'organization' && <OrganizationScreen />}
        {active === 'training' && <TrainingScreen />}
        {active === 'safety' && <SafetyScreen />}
        {active === 'gate-history' && <GateHistoryScreen />}
      </main>
    </Grid>
  )
}
