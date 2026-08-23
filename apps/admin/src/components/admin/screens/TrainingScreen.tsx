import { ScreenHeading } from './PeopleScreen'

const COURSES = [
  ['EDU-SPECIAL-DG', '위험물 취급 특별교육', '위험물 취급 작업', '16시간', '38 / 41'],
  ['EDU-MSDS-1263', 'MSDS 교육 · 도료', 'UN 1263 대상 작업', '배치 전', '24 / 26'],
  ['EDU-MSDS-1993', 'MSDS 교육 · 세정제', 'UN 1993 대상 작업', '배치 전', '19 / 19'],
  ['EDU-REGULAR-H1', '정기 안전보건교육', '현장 작업자', '반기', '87 / 92'],
]

export function TrainingScreen() {
  return (
    <div className="admin-screen">
      <ScreenHeading code="EDUCATION ELIGIBILITY" title="안전교육" description="교육과정, 대상규칙, 이수시간과 작업 투입 가능 여부를 관리합니다." action="교육 등록" />
      <section className="education-summary"><p><strong>오늘 작업 투입 전 확인</strong> 교육 미충족 2명 · 만료 또는 이수시간 부족</p><button type="button">대상자 보기</button></section>
      <div className="education-layout">
        <section>
          <div className="course-head course-grid"><span>코드</span><span>교육명</span><span>대상 규칙</span><span>충족 기준</span><span>충족 인원</span></div>
          {COURSES.map((course) => <button className="course-row course-grid" key={course[0]} type="button">{course.map((value, index) => <span className={index === 0 ? 'mono' : ''} key={`${course[0]}-${index}`}>{value}</span>)}</button>)}
        </section>
        <aside><span className="admin-overline">ELIGIBILITY CHECK</span><h2>작업 배정 전 판정</h2><dl><div><dt>작업</dt><dd>WB-260823-05</dd></div><div><dt>대상자</dt><dd>김태완</dd></div><div><dt>요구교육</dt><dd>특별교육·MSDS</dd></div><div><dt>판정</dt><dd>투입 불가</dd></div><div><dt>사유</dt><dd>MSDS 교육 미이수</dd></div></dl><button type="button">배정에서 제외</button></aside>
      </div>
    </div>
  )
}
