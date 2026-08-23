'use client'

import { Button as UiButton } from '@devup-ui/react'


import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

export function SafetyScreen() {
  const [stopped, setStopped] = useState(false)
  return (
    <div className="admin-screen">
      <ScreenHeading code="SAFETY CONTROL" title="지침·작업중지" description="작업별 안전지침과 현장 작업중지 상태를 관리합니다." action="지침 등록" />
      <div className="safety-layout">
        <section className="instruction-ledger">
          <header><div><span className="admin-overline">ACTIVE INSTRUCTIONS · 03</span><h2>오늘 적용 지침</h2></div><UiButton type="button">현장 알림 작성</UiButton></header>
          <article><time>05:40</time><div><strong>혼재 인화성 액체 취급 지침</strong><p>WB-260823-03 · CFS B-3 · VERSION 4</p></div><span>확인 4 / 6</span></article>
          <article><time>06:10</time><div><strong>위험화물 검수 작업 지침</strong><p>WB-260823-02 · DG 보관소 A · VERSION 2</p></div><span>확인 3 / 4</span></article>
          <article><time>07:00</time><div><strong>강풍 대비 장비 고정 안내</strong><p>CFS·야드 작업 3건 · 14:00 적용</p></div><span>예약</span></article>
        </section>
        <aside className="work-stop">
          <span className="admin-overline">WORK STOP CONTROL</span><h2>{stopped ? 'CFS B-3 작업중지' : '작업중지 없음'}</h2><p>{stopped ? '누출 의심 신고 · 10:24 발령' : '현재 관리자가 발령한 작업중지가 없습니다.'}</p>
          <label>대상 구역<select defaultValue="CFS B-3"><option>CFS B-3</option><option>DG 보관소 A</option><option>전 구역</option></select></label>
          <label>사유<textarea defaultValue="현장 이상상태 확인을 위한 일시 중지" /></label>
          <UiButton onClick={() => setStopped((value) => !value)} type="button">{stopped ? '작업중지 해제' : '작업중지 발령'}</UiButton>
          <small>발령 즉시 준비 완료 여부와 관계없이 게이트 판정이 차단됩니다.</small>
        </aside>
      </div>
    </div>
  )
}
