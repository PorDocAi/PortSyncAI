'use client'

import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

const EVENTS = [
  ['GE-260823-091', '10:18:42', '1부두 정문', '윤서진', 'WB-260823-03', 'PASS', '모든 조건 충족'],
  ['GE-260823-090', '10:16:05', '1부두 정문', '김태완', 'WB-260823-05', 'BLOCK', 'EDUCATION_MISSING'],
  ['GE-260823-089', '10:12:31', '2부두 동문', '박해원', 'WB-260823-02', 'PASS', '모든 조건 충족'],
  ['GE-260823-088', '10:09:18', '1부두 정문', '문지훈', 'WB-260823-03', 'BLOCK', 'PPE_INCOMPLETE'],
]

export function GateHistoryScreen() {
  const [selected, setSelected] = useState(EVENTS[1])
  return (
    <div className="admin-screen">
      <ScreenHeading code="GATE EVENTS · TODAY" title="게이트 판정 이력" description="통과·차단 결과와 당시 사용된 판정근거 버전을 조회합니다." />
      <div className="admin-toolbar"><input placeholder="작업자·사번·작업 ID 검색" /><button type="button">결과 전체</button><button type="button">게이트 전체</button></div>
      <div className="gate-history-layout">
        <section>
          <div className="event-head event-grid"><span>시각</span><span>게이트</span><span>작업자</span><span>작업</span><span>결과</span><span>판정 사유</span></div>
          {EVENTS.map((event) => <button className={selected[0] === event[0] ? 'event-row event-grid is-selected' : 'event-row event-grid'} key={event[0]} onClick={() => setSelected(event)} type="button">{event.slice(1).map((value, index) => <span className={index === 0 || index === 3 ? 'mono' : ''} key={`${event[0]}-${index}`}>{value}</span>)}</button>)}
        </section>
        <aside><span className="admin-overline">{selected[0]}</span><h2>{selected[5]}</h2><p>{selected[3]} · {selected[2]}</p><dl><div><dt>작업 배정</dt><dd>유효</dd></div><div><dt>교육</dt><dd>{selected[5] === 'BLOCK' ? '미충족' : '충족'}</dd></div><div><dt>지침</dt><dd>확인 완료</dd></div><div><dt>보호구</dt><dd>3 / 3</dd></div><div><dt>작업중지</dt><dd>없음</dd></div><div><dt>최종 사유</dt><dd>{selected[6]}</dd></div></dl><button type="button">감사로그 원문 보기</button></aside>
      </div>
    </div>
  )
}
