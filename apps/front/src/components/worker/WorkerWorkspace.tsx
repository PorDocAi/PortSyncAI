'use client'

import { Box, Button as UiButton, Grid, Input as UiInput } from '@devup-ui/react'

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

type WorkerView = 'today' | 'preparation' | 'alerts' | 'history' | 'profile'

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
  const [activeView, setActiveView] = useState<WorkerView>('today')
  const selected = useMemo(
    () => WORKS.find((work) => work.id === selectedId) ?? WORKS[0],
    [selectedId],
  )

  return (
    <Grid className="worker-stage">
      <Box className="worker-app">
        <header className="worker-header">
          <div className="wordmark">
            <span aria-hidden="true">PS</span>
            <strong>PortSyncAI</strong>
          </div>
          <div className="worker-header__meta">
            <UiButton className="header-alert" onClick={() => setActiveView('alerts')} type="button">
              알림 <strong>3</strong>
            </UiButton>
            <span>윤서진</span>
            <Link href="/signin">로그아웃</Link>
          </div>
        </header>

        <main className={activeView === 'today' ? '' : 'is-hidden'}>
          <section className="worker-title">
            <p className="overline">2026.08.23 · 주간조</p>
            <h1>오늘의 작업</h1>
            <p>배정된 작업을 확인한 뒤 준비 절차를 시작하세요.</p>
          </section>

          <UiButton className="field-bulletin" aria-label="기상·긴급 알림 열기" onClick={() => setActiveView('alerts')} type="button">
            <div>
              <span className="field-bulletin__label">기상 주의</span>
              <time>14:00–18:00</time>
            </div>
            <p><strong>오후 강풍 예보</strong> CFS·야드 작업은 장비 고정 상태를 재확인합니다.</p>
            <span className="field-bulletin__link">영향 작업과 대응 확인 →</span>
          </UiButton>

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
                <UiButton
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
                </UiButton>
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
              <UiButton className="text-button" onClick={() => setDetailsOpen((open) => !open)} type="button">
                {detailsOpen ? '화물 상세 접기' : '화물 상세 보기'}
              </UiButton>
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
            <UiButton
              disabled={selected.education !== '충족'}
              onClick={() => setActiveView('preparation')}
              type="button"
            >
              {selected.education === '충족' ? '이 작업 준비 시작' : '교육 확인 후 시작 가능'}
            </UiButton>
          </section>
        </main>

        {activeView === 'preparation' && <PreparationScreen work={selected} />}
        {activeView === 'alerts' && <AlertsScreen onBack={() => setActiveView('today')} />}
        {activeView === 'history' && <HistoryScreen />}
        {activeView === 'profile' && <ProfileScreen />}

        <nav className="worker-nav" aria-label="작업자 메뉴">
          <UiButton className={activeView === 'today' ? 'is-current' : ''} onClick={() => setActiveView('today')} type="button"><span>오늘 작업</span></UiButton>
          <UiButton className={activeView === 'preparation' ? 'is-current' : ''} onClick={() => setActiveView('preparation')} type="button"><span>준비 절차</span></UiButton>
          <UiButton className={activeView === 'history' ? 'is-current' : ''} onClick={() => setActiveView('history')} type="button"><span>통과 기록</span></UiButton>
          <UiButton className={activeView === 'profile' ? 'is-current' : ''} onClick={() => setActiveView('profile')} type="button"><span>내 정보</span></UiButton>
        </nav>
      </Box>
    </Grid>
  )
}

type PreparationStep = 'education' | 'instruction' | 'ppe' | 'gate'

const PREPARATION_STEPS: { id: PreparationStep; label: string; state: string }[] = [
  { id: 'education', label: '안전교육 적격성', state: '충족' },
  { id: 'instruction', label: '당일 안전지침', state: '확인 전' },
  { id: 'ppe', label: '필수 보호구', state: '1 / 3 확인' },
  { id: 'gate', label: '게이트 준비', state: '대기' },
]

function PreparationScreen({ work }: { work: Work }) {
  const [step, setStep] = useState<PreparationStep>('education')
  const [instructionRead, setInstructionRead] = useState(false)
  const [scanned, setScanned] = useState<string[]>(['방폭형 안전화'])
  const [educationOpen, setEducationOpen] = useState(false)
  const [educationPlaying, setEducationPlaying] = useState(false)
  const [exceptionOpen, setExceptionOpen] = useState(false)
  const [exceptionReason, setExceptionReason] = useState('')
  const [exceptionMemo, setExceptionMemo] = useState('')
  const [exceptionSubmitted, setExceptionSubmitted] = useState(false)
  const [gateReady, setGateReady] = useState(false)

  const isStepAvailable = (target: PreparationStep) => {
    if (target === 'education') return true
    if (target === 'instruction') return work.education === '충족'
    if (target === 'ppe') return instructionRead
    return instructionRead && scanned.length === 3
  }

  const scan = (item: string) => {
    setScanned((items) => items.includes(item) ? items : [...items, item])
  }

  return (
    <main className="preparation-screen">
      <header className="flow-header">
        <p className="overline">{work.code}</p>
        <h1>작업 전 준비</h1>
        <p>{work.place} · {work.title}</p>
      </header>

      <ol className="flow-index" aria-label="준비 단계">
        {PREPARATION_STEPS.map((item, index) => {
          const available = isStepAvailable(item.id)
          return (
          <li className={`${step === item.id ? 'is-current' : ''}${available ? '' : ' is-locked'}`} key={item.id}>
            <UiButton disabled={!available} onClick={() => setStep(item.id)} type="button">
              <span>{String(index + 1).padStart(2, '0')}</span>
              <strong>{item.label}</strong>
              <small>
                {item.id === 'instruction' && instructionRead
                  ? '확인 완료'
                  : item.id === 'ppe'
                    ? `${scanned.length} / 3 확인`
                    : item.id === 'gate' && gateReady
                      ? '준비 완료'
                      : available ? item.state : '이전 단계 필요'}
              </small>
            </UiButton>
          </li>
        )})}
      </ol>

      <section className="step-panel">
        {step === 'education' && (
          <>
            <StepHeading number="01" title="안전교육 적격성" description="법정교육을 충족한 작업자만 다음 준비 단계로 이동할 수 있습니다." />
            <div className="eligibility-result">
              <p>판정</p>
              <strong>이 작업에 투입 가능</strong>
              <span>2026.08.23 06:12 자동 판정</span>
            </div>
            <div className="plain-table">
              <div><span>위험물 취급 특별교육</span><strong>16시간 충족</strong></div>
              <div><span>MSDS 교육 · 대상물질</span><strong>PAINT / SOLVENT</strong></div>
              <div><span>기초 안전보건교육</span><strong>정기교육 충족</strong></div>
            </div>
            <UiButton className="inline-disclosure" onClick={() => setEducationOpen((open) => !open)} type="button">
              {educationOpen ? '교육 상세 접기' : '교육 이력·내용 확인'}
            </UiButton>
            {educationOpen && (
              <section className="education-detail" aria-label="위험물 취급 특별교육 상세">
                <header>
                  <div><span>필수교육 01</span><strong>위험물 취급 특별교육</strong></div>
                  <small>유효 · 2028.05.19까지</small>
                </header>
                <div className={educationPlaying ? 'education-player is-playing' : 'education-player'}>
                  <p>교육 영상 · 25분</p>
                  <strong>{educationPlaying ? '재생 중 · 12:34 / 25:00' : 'MSDS와 개인보호구 기본 절차'}</strong>
                  <UiButton onClick={() => setEducationPlaying((playing) => !playing)} type="button">
                    {educationPlaying ? '일시정지' : '재생 시연'}
                  </UiButton>
                </div>
                <dl>
                  <div><dt>이수일</dt><dd>2026.05.20</dd></div>
                  <div><dt>수료 조건</dt><dd>영상 100% · 평가 80점</dd></div>
                  <div><dt>이수 결과</dt><dd>92점 · 수료</dd></div>
                </dl>
              </section>
            )}
            <UiButton className="panel-action" onClick={() => setStep('instruction')} type="button">다음 · 안전지침 확인</UiButton>
          </>
        )}

        {step === 'instruction' && (
          <>
            <StepHeading number="02" title="당일 안전지침" description="현재 작업과 화물에 적용되는 최신 지침입니다." />
            <div className="document-meta">
              <span>SI-260823-CFS-03 · VERSION 4</span>
              <time>08.23 05:40 확정</time>
            </div>
            <article className="instruction-copy">
              <h2>혼재 인화성 액체 취급 지침</h2>
              <h3>작업 전</h3>
              <p>용기 밀폐 상태와 누출 흔적을 확인하고, 점화원이 있는 장비는 작업구역 밖으로 이동합니다.</p>
              <h3>작업 중</h3>
              <p>환기 상태를 유지하고 정전기 발생 가능성이 있는 도구를 사용하지 않습니다. 이상 냄새나 누출을 발견하면 즉시 작업을 중지합니다.</p>
              <h3>기상 주의</h3>
              <p>14시 이후 강풍이 예상됩니다. 적치물과 이동식 장비의 고정 상태를 재확인합니다.</p>
            </article>
            <label className="acknowledge-row">
              <UiInput checked={instructionRead} onChange={(event) => setInstructionRead(event.target.checked)} type="checkbox" />
              <span>지침 전체 내용을 읽고 작업 시 준수하겠습니다.</span>
            </label>
            <UiButton className="panel-action" disabled={!instructionRead} onClick={() => setStep('ppe')} type="button">확인 기록 후 보호구 준비</UiButton>
          </>
        )}

        {step === 'ppe' && (
          <>
            <StepHeading number="03" title="필수 보호구" description="MSDS 8항과 작업유형을 기준으로 확정된 목록입니다." />
            <div className="ppe-list">
              {['방폭형 안전화', '내화학 장갑', '보안경'].map((item, index) => {
                const complete = scanned.includes(item)
                return (
                  <div className="ppe-row" key={item}>
                    <span>{String(index + 1).padStart(2, '0')}</span>
                    <div><strong>{item}</strong><small>{index === 1 ? 'HAND · 화학물질 투과 저항' : index === 2 ? 'EYE · 측면 보호' : 'FOOT · 정전기 방지'}</small></div>
                    <UiButton disabled={complete} onClick={() => scan(item)} type="button">{complete ? '확인됨' : 'NFC 읽기'}</UiButton>
                  </div>
                )
              })}
            </div>
            <div className="nfc-note"><strong>NFC 입력 규칙</strong><p>태그 토큰만 전송하며 장비 종류·소유자·사용중지 여부는 서버에서 판정합니다.</p></div>
            <UiButton className="exception-toggle" onClick={() => setExceptionOpen((open) => !open)} type="button">
              보호구 문제 신고·예외 승인 요청
            </UiButton>
            {exceptionOpen && (
              <section className="exception-form" aria-label="예외 승인 요청">
                {exceptionSubmitted ? (
                  <div className="exception-result">
                    <span>승인 대기</span>
                    <strong>관리자 검토 전까지 게이트 통과가 제한됩니다.</strong>
                    <p>요청번호 EX-260823-014 · 대체 보호구 또는 현장 조치가 확인되어야 합니다.</p>
                  </div>
                ) : (
                  <>
                    <p>예외 승인은 교육 미이수나 보호구 미착용을 면제하지 않습니다. 장비 교체·대체품 확인이 필요한 경우에만 요청하세요.</p>
                    <label>
                      <span>요청 사유</span>
                      <select value={exceptionReason} onChange={(event) => setExceptionReason(event.target.value)}>
                        <option value="">선택해 주세요</option>
                        <option value="damaged">태그 장비 파손·사용중지</option>
                        <option value="replacement">대체 보호구 확인 필요</option>
                        <option value="personal">개인 보호구 등록 요청</option>
                      </select>
                    </label>
                    <label>
                      <span>현장 메모</span>
                      <textarea maxLength={300} onChange={(event) => setExceptionMemo(event.target.value)} placeholder="장비명과 현재 상황을 입력하세요." value={exceptionMemo} />
                    </label>
                    <UiButton disabled={!exceptionReason || exceptionMemo.trim().length < 5} onClick={() => setExceptionSubmitted(true)} type="button">관리자에게 검토 요청</UiButton>
                  </>
                )}
              </section>
            )}
            <UiButton className="panel-action" disabled={scanned.length < 3} onClick={() => setStep('gate')} type="button">게이트 준비상태 확인</UiButton>
          </>
        )}

        {step === 'gate' && (
          <>
            <StepHeading number="04" title="게이트 준비" description="사원증 태깅 시 아래 조건을 서버에서 다시 검증합니다." />
            <div className="gate-checks">
              <div><span>작업 배정</span><strong>유효</strong></div>
              <div><span>안전교육</span><strong>충족</strong></div>
              <div><span>안전지침</span><strong>{instructionRead ? '확인 완료' : '확인 필요'}</strong></div>
              <div><span>필수 보호구</span><strong>{scanned.length === 3 ? '3종 완료' : `${3 - scanned.length}종 미확인`}</strong></div>
              <div><span>작업중지</span><strong>없음</strong></div>
            </div>
            {gateReady ? (
              <div className="gate-ready-result">
                <span>READY · 07:36:18</span>
                <strong>게이트 이동 가능</strong>
                <p>1부두 정문 태블릿에서 사원증을 태그하면 서버가 모든 조건을 다시 판정합니다.</p>
                <dl>
                  <div><dt>작업</dt><dd>{work.code}</dd></div>
                  <div><dt>준비기록</dt><dd>PREP-260823-0031</dd></div>
                </dl>
              </div>
            ) : (
              <div className="gate-direction">
                <span>GATE</span>
                <strong>1부두 정문</strong>
                <p>준비 완료를 기록한 뒤 게이트 리더에 사원증을 태그해 주세요.</p>
              </div>
            )}
            <UiButton className="panel-action" disabled={!instructionRead || scanned.length < 3 || gateReady} onClick={() => setGateReady(true)} type="button">
              {gateReady ? '준비 완료 기록됨' : '게이트 통과 준비 완료'}
            </UiButton>
          </>
        )}
      </section>
    </main>
  )
}

function AlertsScreen({ onBack }: { onBack: () => void }) {
  const [acknowledged, setAcknowledged] = useState<string[]>([])
  const notices = [
    { id: 'wind', level: '주의', time: '오늘 07:10', title: '오후 강풍 예보', body: '14시 이후 순간풍속 증가가 예상됩니다. CFS·야드 작업은 적치물과 이동식 장비 고정을 재확인합니다.', source: '기상 연동' },
    { id: 'crane', level: '긴급', time: '오늘 08:40', title: '3부두 크레인 점검 중', body: '점검 종료 공지 전까지 3부두 작업자 출입과 장비 접근을 금지합니다.', source: '안전관리자' },
    { id: 'wave', level: '주의', time: '어제 14:20', title: '높은 파고 주의', body: '해상 작업자는 구명조끼 체결 상태와 작업구역 통제선을 확인하세요.', source: '기상 연동' },
  ]

  return (
    <main className="alerts-screen">
      <header>
        <UiButton onClick={onBack} type="button">← 오늘 작업</UiButton>
        <p className="overline">BUSAN PORT · LIVE</p>
        <h1>기상·긴급 알림</h1>
        <p>현재 작업에 영향을 주는 공지만 우선 표시합니다.</p>
      </header>
      <section className="weather-summary">
        <div><span>현재</span><strong>27°C</strong></div>
        <dl>
          <div><dt>풍속</dt><dd>3m/s</dd></div>
          <div><dt>습도</dt><dd>62%</dd></div>
          <div><dt>기준</dt><dd>08:55</dd></div>
        </dl>
      </section>
      <section className="notice-list" aria-label="안전 알림 목록">
        {notices.map((notice) => {
          const done = acknowledged.includes(notice.id)
          return (
            <article className={notice.level === '긴급' ? 'is-urgent' : ''} key={notice.id}>
              <header><span>{notice.level}</span><time>{notice.time}</time></header>
              <h2>{notice.title}</h2>
              <p>{notice.body}</p>
              <footer><small>{notice.source}</small><UiButton disabled={done} onClick={() => setAcknowledged((items) => [...items, notice.id])} type="button">{done ? '확인 기록됨' : '내용 확인'}</UiButton></footer>
            </article>
          )
        })}
      </section>
    </main>
  )
}

function StepHeading({ number, title, description }: { number: string; title: string; description: string }) {
  return (
    <header className="step-heading">
      <span>{number}</span>
      <div><h2>{title}</h2><p>{description}</p></div>
    </header>
  )
}

function HistoryScreen() {
  return (
    <main className="simple-screen">
      <header><p className="overline">LAST 90 DAYS</p><h1>게이트 통과 기록</h1><p>통과와 차단 결과, 판정 사유를 확인합니다.</p></header>
      <div className="history-list">
        <div><time>08.23<br />07:42</time><p><strong>1부두 정문 · 통과</strong><span>WB-260823-03 · 정기 출입</span></p><small>PASS</small></div>
        <div><time>08.22<br />08:05</time><p><strong>1부두 정문 · 통과</strong><span>WB-260822-03 · 정기 출입</span></p><small>PASS</small></div>
        <div><time>08.21<br />07:51</time><p><strong>1부두 정문 · 차단</strong><span>필수 보호구 1종 미확인</span></p><small>BLOCK</small></div>
      </div>
    </main>
  )
}

function ProfileScreen() {
  return (
    <main className="simple-screen">
      <header><p className="overline">EMP-240031</p><h1>윤서진</h1><p>CFS운영팀 · 화물 분류 작업자</p></header>
      <dl className="profile-list">
        <div><dt>계정 상태</dt><dd>재직 · 사용 가능</dd></div>
        <div><dt>소속 팀</dt><dd>CFS 2조</dd></div>
        <div><dt>등록 사원증</dt><dd>•••• 8A21</dd></div>
        <div><dt>선호 언어</dt><dd>한국어</dd></div>
      </dl>
      <UiButton className="secondary-action" type="button">비밀번호 변경</UiButton>
    </main>
  )
}
