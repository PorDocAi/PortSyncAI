'use client'

import Link from 'next/link'
import { useMemo, useState } from 'react'

type Work = {
  id: string
  code: string
  title: string
  place: string
  time: string
  team: string
  cargo: string
  education: '충족' | '확인 필요'
}

const WORKS: Work[] = [
  {
    id: 'wb-03',
    code: 'WB-260823-03',
    title: 'CFS 적출·분류',
    place: 'CFS B-3',
    time: '08:00–17:00',
    team: 'CFS 2조 · 6명',
    cargo: 'CONT-260823-014 · 혼재화물 2종',
    education: '충족',
  },
  {
    id: 'wb-05',
    code: 'WB-260823-05',
    title: '위험화물 검수',
    place: 'DG 보관소 A',
    time: '13:30–16:30',
    team: 'DG 전담조 · 4명',
    cargo: 'CONT-260823-021 · Class 8',
    education: '확인 필요',
  },
]

export function WorkerWorkspace() {
  const [selectedId, setSelectedId] = useState(WORKS[0].id)
  const [detailsOpen, setDetailsOpen] = useState(false)
  const selected = useMemo(
    () => WORKS.find((work) => work.id === selectedId) ?? WORKS[0],
    [selectedId],
  )

  return (
    <div className="worker-stage">
      <div className="worker-app">
        <header className="worker-header">
          <div className="wordmark">
            <span aria-hidden="true">PS</span>
            <strong>PortSyncAI</strong>
          </div>
          <div className="worker-header__meta">
            <span>윤서진</span>
            <Link href="/signin">로그아웃</Link>
          </div>
        </header>

        <main>
          <section className="worker-title">
            <p className="overline">2026.08.23 · 주간조</p>
            <h1>오늘의 작업</h1>
            <p>배정된 작업을 확인한 뒤 준비 절차를 시작하세요.</p>
          </section>

          <section className="field-bulletin" aria-label="현장 공지">
            <div>
              <span className="field-bulletin__label">기상 주의</span>
              <time>14:00–18:00</time>
            </div>
            <p><strong>오후 강풍 예보</strong> CFS·야드 작업은 장비 고정 상태를 재확인합니다.</p>
          </section>

          <section className="work-selector" aria-labelledby="work-list-title">
            <div className="section-title">
              <div>
                <p className="overline">ASSIGNMENTS · 02</p>
                <h2 id="work-list-title">배정 작업</h2>
              </div>
              <span>교육 적격성 사전 확인</span>
            </div>
            <div className="work-list">
              {WORKS.map((work, index) => (
                <button
                  className={work.id === selectedId ? 'work-row is-selected' : 'work-row'}
                  key={work.id}
                  onClick={() => setSelectedId(work.id)}
                  type="button"
                >
                  <span className="work-row__index">{String(index + 1).padStart(2, '0')}</span>
                  <span className="work-row__main">
                    <small>{work.code}</small>
                    <strong>{work.title}</strong>
                    <span>{work.place} · {work.time}</span>
                  </span>
                  <span className={work.education === '충족' ? 'text-status' : 'text-status is-blocked'}>
                    교육 {work.education}
                  </span>
                </button>
              ))}
            </div>
          </section>

          <section className="selected-work" aria-labelledby="selected-work-title">
            <div className="selected-work__code">
              <small>{selected.code}</small>
              <strong>{selected.place}</strong>
            </div>
            <div className="selected-work__body">
              <p className="overline">SELECTED WORK</p>
              <h2 id="selected-work-title">{selected.title}</h2>
              <dl>
                <div><dt>시간</dt><dd>{selected.time}</dd></div>
                <div><dt>작업팀</dt><dd>{selected.team}</dd></div>
                <div><dt>대상화물</dt><dd>{selected.cargo}</dd></div>
                <div><dt>교육</dt><dd>{selected.education}</dd></div>
              </dl>
              <button className="text-button" onClick={() => setDetailsOpen((open) => !open)} type="button">
                {detailsOpen ? '화물 상세 접기' : '화물 상세 보기'}
              </button>
              {detailsOpen && (
                <div className="cargo-detail">
                  <div><span>01</span><p><strong>도료</strong> UN 1263 · Class 3</p></div>
                  <div><span>02</span><p><strong>세정제</strong> UN 1993 · Class 3</p></div>
                  <p>MSDS 검수 완료 · 보호구 조건 확정 v3</p>
                </div>
              )}
            </div>
          </section>

          <section className="start-section">
            <p>교육 적격성을 다시 확인한 뒤 준비 절차가 열립니다.</p>
            <button disabled={selected.education !== '충족'} type="button">
              {selected.education === '충족' ? '이 작업 준비 시작' : '교육 확인 후 시작 가능'}
            </button>
          </section>
        </main>

        <nav className="worker-nav" aria-label="작업자 메뉴">
          <button className="is-current" type="button"><span>오늘 작업</span></button>
          <button type="button"><span>준비 절차</span></button>
          <button type="button"><span>통과 기록</span></button>
          <button type="button"><span>내 정보</span></button>
        </nav>
      </div>
    </div>
  )
}
