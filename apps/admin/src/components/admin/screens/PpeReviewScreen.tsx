'use client'

import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

const REQUIREMENTS = [
  ['RESPIRATORY', '유기화합물용 방독마스크', '필수'],
  ['HAND', '내화학 장갑 · 투과저항 확인', '필수'],
  ['EYE', '측면 보호형 보안경', '필수'],
  ['BODY', '내화학 보호복', '조건부'],
]

export function PpeReviewScreen() {
  const [confirmed, setConfirmed] = useState(false)
  return (
    <div className="admin-screen">
      <ScreenHeading code="MSDS REVIEW · 1 / 4" title="보호구 조건 검수" description="MSDS 8항 원문과 정규화 결과를 비교해 작업 기준을 확정합니다." />
      <div className="review-context"><span className="mono">DOC-260823-041</span><strong>paint_msds_ko.pdf</strong><span>CONT-260823-014 · 도료 (PAINT)</span><button type="button">다른 문서 선택</button></div>
      <div className="review-bench">
        <section className="source-pane">
          <header><span className="admin-overline">SOURCE · PAGE 6</span><h2>8. 노출방지 및 개인보호구</h2></header>
          <article><p><b>가. 화학물질의 노출기준</b><br />해당 없음</p><p><b>나. 적절한 공학적 관리</b><br />국소배기장치를 설치하고 적절한 환기를 유지할 것.</p><p><b>다. 개인보호구</b><br />호흡기 보호: 유기화합물용 정화통을 장착한 방독마스크를 착용할 것.<br /><br />눈 보호: 측면 보호판이 있는 보안경을 착용할 것.<br /><br />손 보호: 화학물질에 견디는 보호장갑을 착용할 것.<br /><br />신체 보호: 필요 시 적절한 내화학성 보호의를 착용할 것.</p></article>
          <footer>원문 위치 8-다 · 텍스트 추출 신뢰도 94%</footer>
        </section>
        <section className="normalized-pane">
          <header><span className="admin-overline">NORMALIZED REQUIREMENTS</span><h2>작업자 안내 조건</h2></header>
          <div className="requirement-list">
            {REQUIREMENTS.map((item, index) => <div key={item[0]}><span>{String(index + 1).padStart(2, '0')}</span><p><small>{item[0]}</small><strong>{item[1]}</strong></p><select defaultValue={item[2]}><option>필수</option><option>조건부</option><option>제외</option></select></div>)}
          </div>
          <label className="review-note">검수 메모<textarea defaultValue="혼재 화물 병합 시 RESPIRATORY와 HAND 성능조건을 우선 적용." /></label>
        </section>
        <aside className="review-decision">
          <span className="admin-overline">DECISION</span><h2>{confirmed ? '조건 확정됨' : '관리자 검수 필요'}</h2>
          <dl><div><dt>문서 식별</dt><dd>일치</dd></div><div><dt>DGD 대조</dt><dd>UN 1263 / Class 3</dd></div><div><dt>필수 조건</dt><dd>3종</dd></div><div><dt>조건부</dt><dd>1종</dd></div></dl>
          <button onClick={() => setConfirmed(true)} type="button">{confirmed ? '확정 기록 저장됨' : '보호구 조건 확정'}</button>
          <p>확정 후 연결된 작업의 준비조건 버전이 갱신됩니다.</p>
        </aside>
      </div>
    </div>
  )
}
