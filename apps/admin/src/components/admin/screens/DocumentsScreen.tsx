'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'


import { useState } from 'react'

import { ScreenHeading } from './PeopleScreen'

const DOCUMENTS = [
  ['DOC-260823-041', 'MSDS', 'CONT-260823-014', 'paint_msds_ko.pdf', '추출 완료', '검수 대기'],
  ['DOC-260823-040', 'DGD', 'CONT-260823-014', 'DGD_014.pdf', '추출 완료', '확정'],
  ['DOC-260823-039', 'C/I', 'CONT-260823-014', 'CI_014.pdf', '추출 완료', '확정'],
  ['DOC-260823-038', 'MSDS', 'CONT-260823-021', 'cleaner_msds.pdf', '문자인식 실패', '조치 필요'],
  ['DOC-260823-037', 'DGD', 'CONT-260823-021', 'DGD_021.pdf', '처리 중', '대기'],
]

export function DocumentsScreen() {
  const [selected, setSelected] = useState(DOCUMENTS[0])
  return (
    <div className="admin-screen">
      <ScreenHeading code="DOCUMENT REGISTER" title="화물 문서" description="DGD·C/I·MSDS 원본과 처리·검수 상태를 관리합니다." action="문서 등록" />
      <section className="document-intake">
        <div><span className="admin-overline">NEW DOCUMENT</span><strong>파일을 선택하거나 이 영역에 놓으세요.</strong><small>PDF·이미지 · 최대 20MB · 원본 해시 저장</small></div>
        <label>문서 종류<select defaultValue="MSDS"><option>MSDS</option><option>DGD</option><option>C/I</option></select></label>
        <label>컨테이너<UiInput defaultValue="CONT-260823-014" /></label>
        <UiButton type="button">파일 선택</UiButton>
      </section>
      <div className="document-workspace">
        <section className="document-register">
          <div className="document-head document-grid"><span>문서 ID</span><span>종류</span><span>컨테이너</span><span>파일명</span><span>처리</span><span>검수</span></div>
          {DOCUMENTS.map((item) => <UiButton className={item[0] === selected[0] ? 'document-row document-grid is-selected' : 'document-row document-grid'} key={item[0]} onClick={() => setSelected(item)} type="button">{item.map((value, index) => <span className={index === 0 ? 'mono' : ''} key={`${item[0]}-${index}`}>{value}</span>)}</UiButton>)}
        </section>
        <aside className="document-inspector">
          <div><span className="admin-overline">{selected[0]}</span><h2>{selected[1]}</h2><p>{selected[3]}</p></div>
          <dl><div><dt>연결 컨테이너</dt><dd>{selected[2]}</dd></div><div><dt>처리 상태</dt><dd>{selected[4]}</dd></div><div><dt>검수 상태</dt><dd>{selected[5]}</dd></div><div><dt>파일 무결성</dt><dd>SHA-256 확인</dd></div></dl>
          <UiButton type="button">원문·추출 결과 열기</UiButton>
        </aside>
      </div>
    </div>
  )
}
