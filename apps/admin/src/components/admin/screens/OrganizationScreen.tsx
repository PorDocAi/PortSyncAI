import { Button as UiButton } from '@devup-ui/react'
import { ScreenHeading } from './PeopleScreen'

const DEPARTMENTS = [
  ['CFS', 'CFS운영팀', '컨테이너 적출·분류 및 화물 이동', '18명'],
  ['SAFETY', '안전관리팀', '작업 전 안전조건과 현장 작업중지 관리', '6명'],
  ['DGR', 'DG운영팀', '위험화물 검수·보관·취급', '11명'],
  ['DOC', '문서관리팀', '화물문서 등록·추출 결과 검수', '4명'],
]

const JOBS = [
  ['CFS-01', '화물 분류 작업자', 'CFS', '특별교육 대상'],
  ['SAF-01', '안전 관리자', 'SAFETY', '관리 권한'],
  ['DGR-02', '위험물 취급자', 'DGR', '특별교육 대상'],
  ['DOC-01', '화물 문서 담당자', 'DOC', '문서 검수 권한'],
]

export function OrganizationScreen() {
  return (
    <div className="admin-screen">
      <ScreenHeading code="ORGANIZATION" title="부서·직무" description="작업배정과 교육대상 규칙에 사용하는 조직 기준정보입니다." action="기준정보 등록" />
      <div className="split-ledger">
        <section>
          <header><div><span className="admin-overline">DEPARTMENTS</span><h2>부서</h2></div><UiButton type="button">추가</UiButton></header>
          <div className="ledger-head"><span>코드</span><span>부서명·업무</span><span>인원</span></div>
          {DEPARTMENTS.map((item) => <UiButton className="ledger-row" key={item[0]} type="button"><span className="mono">{item[0]}</span><span><strong>{item[1]}</strong><small>{item[2]}</small></span><span>{item[3]}</span></UiButton>)}
        </section>
        <section>
          <header><div><span className="admin-overline">JOB ROLES</span><h2>직무</h2></div><UiButton type="button">추가</UiButton></header>
          <div className="ledger-head"><span>코드</span><span>직무명·소속</span><span>정책</span></div>
          {JOBS.map((item) => <UiButton className="ledger-row" key={item[0]} type="button"><span className="mono">{item[0]}</span><span><strong>{item[1]}</strong><small>{item[2]}</small></span><span>{item[3]}</span></UiButton>)}
        </section>
      </div>
    </div>
  )
}
