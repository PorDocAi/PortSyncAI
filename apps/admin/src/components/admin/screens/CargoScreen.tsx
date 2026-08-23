'use client'

import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

const CONTAINERS = [
  ['CONT-260823-014', '08.23 07:30', 'CFS B-3', '2종', 'MSDS 검수 대기'],
  ['CONT-260823-021', '08.23 12:40', 'DG 보관소 A', '1종', '문서 조치 필요'],
  ['CONT-260823-026', '08.24 09:00', 'E-5 야드', '3종', '조건 확정'],
  ['CONT-260824-003', '08.24 10:20', 'CFS A-1', '1종', '문서 수집 중'],
]

export function CargoScreen() {
  const [selected, setSelected] = useState(CONTAINERS[0])
  return (
    <div className="admin-screen">
      <ScreenHeading code="CARGO REGISTER" title="컨테이너·화물" description="컨테이너 안의 복수 화물과 연결 문서를 함께 관리합니다." action="컨테이너 등록" />
      <div className="cargo-workspace">
        <section className="container-ledger">
          <div className="cargo-head cargo-grid"><span>컨테이너</span><span>입고 예정</span><span>작업 구역</span><span>화물</span><span>안전조건</span></div>
          {CONTAINERS.map((item) => <button className={selected[0] === item[0] ? 'cargo-row cargo-grid is-selected' : 'cargo-row cargo-grid'} key={item[0]} onClick={() => setSelected(item)} type="button">{item.map((value, index) => <span className={index === 0 ? 'mono' : ''} key={`${item[0]}-${index}`}>{value}</span>)}</button>)}
        </section>
        <aside className="cargo-inspector">
          <header><span className="admin-overline">CONTAINER</span><h2>{selected[0]}</h2><p>{selected[2]} · {selected[1]}</p></header>
          <section><div className="cargo-item-title"><span>01</span><strong>도료 (PAINT)</strong></div><dl><div><dt>UN No.</dt><dd>UN 1263</dd></div><div><dt>Class</dt><dd>3</dd></div><div><dt>MSDS</dt><dd>검수 대기</dd></div></dl></section>
          <section><div className="cargo-item-title"><span>02</span><strong>세정제 (SOLVENT)</strong></div><dl><div><dt>UN No.</dt><dd>UN 1993</dd></div><div><dt>Class</dt><dd>3</dd></div><div><dt>MSDS</dt><dd>확정</dd></div></dl></section>
          <footer><p>두 화물의 보호구 요구조건은 합집합 후 엄격조건으로 병합됩니다.</p><button type="button">화물 구성 수정</button></footer>
        </aside>
      </div>
    </div>
  )
}
