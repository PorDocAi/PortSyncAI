'use client'

import { Button as UiButton } from '@devup-ui/react'


import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

const WORKS = [
  ['WB-260823-01', '컨테이너 양하', 'E-5 야드', '06:00–14:00', '야드 1조', '확정'],
  ['WB-260823-02', '위험화물 검수', 'DG 보관소 A', '07:30–12:00', 'DG 전담조', '1명 미준비'],
  ['WB-260823-03', 'CFS 적출·분류', 'CFS B-3', '08:00–17:00', 'CFS 2조', '2명 미준비'],
  ['WB-260823-04', '선적 준비', 'F-2 야드', '14:00–22:00', '미배정', '배정 필요'],
]

const MEMBERS = [
  ['EMP-240031', '윤서진', '화물 분류', '교육 충족', '지침 확인 전', '보호구 1 / 3'],
  ['EMP-240036', '최민호', '화물 분류', '교육 충족', '확인 완료', '보호구 3 / 3'],
  ['EMP-240041', '문지훈', 'CFS 반장', '교육 충족', '확인 완료', '보호구 3 / 3'],
  ['EMP-240052', '백수현', '화물 분류', '교육 충족', '확인 완료', '보호구 3 / 3'],
]

export function AssignmentsScreen() {
  const [selected, setSelected] = useState(WORKS[2])
  return (
    <div className="admin-screen">
      <ScreenHeading code="WORK ORDERS · 04" title="작업·팀 배정" description="교육 적격성을 먼저 확인하고 작업자와 팀을 확정합니다." action="새 작업" />
      <div className="assignment-workspace">
        <section className="work-order-list">
          <header><span>오늘 작업</span><UiButton type="button">필터</UiButton></header>
          {WORKS.map((work) => <UiButton className={selected[0] === work[0] ? 'work-order is-selected' : 'work-order'} key={work[0]} onClick={() => setSelected(work)} type="button"><span className="mono">{work[0]}</span><strong>{work[1]}</strong><p>{work[2]} · {work[3]}</p><small>{work[4]} / {work[5]}</small></UiButton>)}
        </section>
        <section className="assignment-editor">
          <header><span className="admin-overline">{selected[0]}</span><h2>{selected[1]}</h2><p>{selected[2]} · {selected[3]}</p></header>
          <div className="assignment-facts"><div><span>컨테이너</span><strong>CONT-260823-014</strong></div><div><span>작업유형</span><strong>CFS 적출·분류</strong></div><div><span>문서 조건</span><strong>검수 완료</strong></div><div><span>교육 규칙</span><strong>특별교육 + MSDS</strong></div></div>
          <div className="team-heading"><div><span className="admin-overline">CFS 2조 · 6명</span><h3>배정 작업자</h3></div><UiButton type="button">적격자 추가</UiButton></div>
          <div className="member-head member-grid"><span>사번·이름</span><span>직무</span><span>교육</span><span>지침</span><span>보호구</span></div>
          {MEMBERS.map((member) => <div className="member-row member-grid" key={member[0]}><span><small className="mono">{member[0]}</small><strong>{member[1]}</strong></span>{member.slice(2).map((value) => <span key={value}>{value}</span>)}</div>)}
        </section>
      </div>
    </div>
  )
}
