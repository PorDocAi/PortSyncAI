'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'


import { useMemo, useState } from 'react'

const PEOPLE = [
  ['EMP-240031', '윤서진', 'CFS운영팀', '화물 분류 작업자', 'WORKER', '재직', '•••• 8A21'],
  ['EMP-240044', '박해원', '안전관리팀', '안전 관리자', 'SAFETY_MANAGER', '재직', '•••• 13F0'],
  ['EMP-240057', '김태완', 'DG운영팀', '위험물 취급자', 'WORKER', '재직', '미등록'],
  ['EMP-240061', '장수민', '시설운영팀', '게이트 운영자', 'SUPERVISOR', '휴직', '•••• C902'],
  ['EMP-240073', '이재경', '문서관리팀', '화물 문서 담당자', 'ADMIN', '재직', '•••• 1E37'],
]

export function PeopleScreen() {
  const [query, setQuery] = useState('')
  const [selected, setSelected] = useState(PEOPLE[0])
  const filtered = useMemo(() => PEOPLE.filter((person) => person.join(' ').includes(query)), [query])

  return (
    <div className="admin-screen">
      <ScreenHeading code="PEOPLE / 05" title="직원 관리" description="계정, 권한, 소속과 사원증 UID를 관리합니다." action="직원 등록" />
      <div className="admin-toolbar"><UiInput onChange={(event) => setQuery(event.target.value)} placeholder="이름·사번·부서 검색" value={query} /><UiButton type="button">부서 전체</UiButton><UiButton type="button">재직상태 전체</UiButton></div>
      <div className="master-detail">
        <section className="data-region">
          <div className="data-head people-grid"><span>사번</span><span>이름</span><span>부서</span><span>직무</span><span>권한</span><span>상태</span></div>
          {filtered.map((person) => (
            <UiButton className={selected[0] === person[0] ? 'data-row people-grid is-selected' : 'data-row people-grid'} key={person[0]} onClick={() => setSelected(person)} type="button">
              {person.slice(0, 6).map((value, index) => <span className={index === 0 ? 'mono' : ''} key={`${person[0]}-${value}`}>{value}</span>)}
            </UiButton>
          ))}
        </section>
        <aside className="detail-rail">
          <div className="detail-rail__identity"><span className="mono">{selected[0]}</span><strong>{selected[1]}</strong><p>{selected[2]} · {selected[3]}</p></div>
          <dl><div><dt>시스템 권한</dt><dd>{selected[4]}</dd></div><div><dt>계정 상태</dt><dd>{selected[5]}</dd></div><div><dt>사원증 UID</dt><dd>{selected[6]}</dd></div><div><dt>초기 비밀번호</dt><dd>변경 완료</dd></div></dl>
          <div className="detail-rail__actions"><UiButton type="button">정보 수정</UiButton><UiButton type="button">사원증 교체</UiButton><UiButton type="button">임시 비밀번호 발급</UiButton></div>
        </aside>
      </div>
    </div>
  )
}

export function ScreenHeading({ code, title, description, action }: { code: string; title: string; description: string; action?: string }) {
  return (
    <header className="screen-heading">
      <div><p className="admin-overline">{code}</p><h1>{title}</h1><p>{description}</p></div>
      {action && <UiButton type="button">{action}</UiButton>}
    </header>
  )
}
