import { Button as UiButton } from '@devup-ui/react'
import { ScreenHeading } from './PeopleScreen'

const RUN_SHEET = [
  ['06:00', 'E-5 야드', 'WB-260823-01', '컨테이너 양하', '야드 1조', '준비 8 / 8'],
  ['07:30', 'DG 보관소 A', 'WB-260823-02', '위험화물 검수', 'DG 전담조', '준비 3 / 4'],
  ['08:00', 'CFS B-3', 'WB-260823-03', 'CFS 적출·분류', 'CFS 2조', '준비 4 / 6'],
  ['13:30', 'DG 보관소 A', 'WB-260823-05', '위험화물 검수', 'DG 전담조', '교육 확인 1'],
  ['14:00', 'F-2 야드', 'WB-260823-04', '선적 준비', '미배정', '팀 배정 필요'],
]

export function OperationsScreen() {
  return (
    <div className="admin-screen">
      <ScreenHeading code="SHIFT 01 · 10:20 CURRENT" title="현장 운영 보드" description="오늘 처리할 작업과 이상 상태를 시간 순서로 확인합니다." action="작업 생성" />
      <section className="ops-bulletin"><span>기상 주의 · 14:00–18:00</span><p><strong>오후 강풍 예보</strong> CFS·야드 작업의 장비 고정 상태를 확인합니다.</p><UiButton type="button">영향 작업 3건</UiButton></section>
      <div className="ops-layout">
        <section className="run-sheet">
          <header><div><span className="admin-overline">오늘 작업 · 시간 순</span><h2>작업 진행표</h2></div><p>전체 5건</p></header>
          <div className="run-head run-grid"><span>시작</span><span>구역</span><span>작업 ID</span><span>작업</span><span>배정팀</span><span>준비상태</span></div>
          {RUN_SHEET.map((row, index) => <UiButton className="run-row run-grid" key={row[2]} type="button"><time>{row[0]}</time><strong>{row[1]}</strong><span className="mono">{row[2]}</span><span>{row[3]}</span><span>{row[4]}</span><span className={index > 2 ? 'needs-action' : ''}>{row[5]}</span></UiButton>)}
        </section>
        <aside className="action-ledger">
          <header><span className="admin-overline">확인 필요 · 4건</span><h2>처리할 일</h2></header>
          <ol>
            <li><span>01</span><div><strong>교육 미충족 작업자</strong><p>WB-260823-05 · 김태완</p></div><UiButton type="button">확인</UiButton></li>
            <li><span>02</span><div><strong>MSDS 검수 대기</strong><p>DOC-260823-041 · CONT-014</p></div><UiButton type="button">검수</UiButton></li>
            <li><span>03</span><div><strong>작업팀 미배정</strong><p>WB-260823-04 · F-2 야드</p></div><UiButton type="button">배정</UiButton></li>
            <li><span>04</span><div><strong>보호구 파손 접수</strong><p>EQ-HAND-0182 · 사용중지</p></div><UiButton type="button">확인</UiButton></li>
          </ol>
        </aside>
      </div>
    </div>
  )
}
