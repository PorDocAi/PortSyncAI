'use client'

import { Box, Button as UiButton, Grid, Input as UiInput } from '@devup-ui/react'
import Link from 'next/link'
import { apiClient } from '@/lib/apiClient'
import { useEffect, useMemo, useState } from 'react'

import { BrandLockup } from '@/components/BrandLockup'
import { scanEquipmentTag, REASON_MESSAGES, type TagResult } from '@/lib/nfc'

type Work = {
  id: string
  /** 서버의 v2_work_assignments PK (태깅 요청에 그대로 사용) */
  assignmentId: number
  code: string
  title: string
  place: string
  time: string
  team: string
  cargo: string
  education: '충족' | '확인 필요'
}

type WorkerView = 'today' | 'preparation' | 'alerts' | 'history' | 'profile'

/** 배정 작업별 필수 보호구 (Class 3 도료 기준, 서버 work_ppe_requirement_snapshots와 동기화) */
const PPE_REQUIREMENTS = [
  { category: 'FOOT', name: '방폭형 안전화', description: 'FOOT · 정전기·스파크 방지' },
  { category: 'HAND', name: '내화학 장갑', description: 'HAND · 화학물질 투과 저항' },
  { category: 'HEAD', name: '안전모', description: 'HEAD · 낙하물·충격 방지' },
  { category: 'BODY', name: '정전기 방지 작업복', description: 'BODY · 정전기 축적 방지' },
] as const

// 서버 응답(GET /work-assignments/my)을 화면 모델로 매핑한다.
type ApiAssignment = {
  work_assignment_id: number
  status: string
  work_id: number
  work_reference: string
  work_status: string
  scheduled_start_at: string
  scheduled_end_at: string | null
}

function toWork(a: ApiAssignment): Work {
  const start = new Date(a.scheduled_start_at)
  const end = a.scheduled_end_at ? new Date(a.scheduled_end_at) : null
  const fmt = (d: Date) => `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
  return {
    id: String(a.work_assignment_id),
    assignmentId: a.work_assignment_id,
    code: a.work_reference,
    title: a.work_reference,
    place: `작업 #${a.work_id}`,
    time: end ? `${fmt(start)}–${fmt(end)}` : fmt(start),
    team: a.status,
    cargo: a.work_reference,
    education: '충족',
  }
}

export function WorkerWorkspace() {
  const [works, setWorks] = useState<Work[]>([])
  const [loadingWorks, setLoadingWorks] = useState(true)
  const [selectedId, setSelectedId] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false
    void (async () => {
      try {
        const res = (await fetch(
          `${process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://127.0.0.1:18090'}/work-assignments/my`,
          { headers: { Authorization: `Bearer ${localStorage.getItem('ps_token') ?? ''}` } },
        ).then((r) => { if (!r.ok) throw new Error(String(r.status)); return r.json() })) as ApiAssignment[]
        if (cancelled) return
        const mapped = res.map(toWork)
        setWorks(mapped)
        setSelectedId(mapped[0]?.id ?? null)
      } catch {
        if (!cancelled) setWorks([])
      } finally {
        if (!cancelled) setLoadingWorks(false)
      }
    })()
    return () => { cancelled = true }
  }, [])
  const [detailsOpen, setDetailsOpen] = useState(false)
  const [activeView, setActiveView] = useState<WorkerView>('today')
  const [weatherAlertOpen, setWeatherAlertOpen] = useState(true)
  const selected = useMemo(
    () => works.find((work) => work.id === selectedId) ?? works[0],
    [works, selectedId],
  )

  return (
    <Grid className="worker-stage">
      <Box className="worker-app">
        <header className="worker-header">
          <BrandLockup />
          <Link href="/gate-terminal" className="gate-link" aria-label="게이트 단말">GATE</Link>
          <div className="worker-header__meta">
            <UiButton
              className="header-alert"
              onClick={() => setActiveView('alerts')}
              type="button"
            >
              알림 <strong>3</strong>
            </UiButton>
            <span>윤서진</span>
            <Link href="/signin">로그아웃</Link>
          </div>
        </header>

        <div className="worker-body">
        <main className={activeView === 'today' ? '' : 'is-hidden'}>
          <section className="worker-title">
            <p className="overline">2026.08.23 · 주간조</p>
            <h1>오늘의 작업</h1>
            <p>배정된 작업을 확인한 뒤 준비 절차를 시작하세요.</p>
          </section>

          <UiButton
            className="field-bulletin"
            aria-label="강풍 예보 상세 열기"
            onClick={() => setWeatherAlertOpen(true)}
            type="button"
          >
            <div>
              <span className="field-bulletin__label">기상 주의</span>
              <time>14:00–18:00</time>
            </div>
            <p>
              <strong>오후 강풍 예보</strong> CFS·야드 작업은 장비 고정 상태를
              재확인합니다.
            </p>
            <span className="field-bulletin__link">
              영향 작업과 대응 확인 →
            </span>
          </UiButton>

          <section aria-labelledby="work-list-title" className="work-selector">
            <div className="section-title">
              <div>
                <p className="overline">ASSIGNMENTS · 02</p>
                <h2 id="work-list-title">배정 작업</h2>
              </div>
              <span>교육 적격성 사전 확인</span>
            </div>
            <div className="work-list">
              {(loadingWorks ? [] : works).map((work, index) => (
                <UiButton
                  key={work.id}
                  className={work.id === selectedId ? 'work-row is-selected' : 'work-row'}
                  onClick={() => setSelectedId(work.id)}
                  type="button"
                >
                  <span className="work-row__index">
                    {String(index + 1).padStart(2, '0')}
                  </span>
                  <span className="work-row__main">
                    <small>{work.code}</small>
                    <strong>{work.title}</strong>
                    <span>
                      {work.place} · {work.time}
                    </span>
                  </span>
                  <span
                    className={
                      work.education === '충족'
                        ? 'text-status'
                        : 'text-status is-blocked'
                    }
                  >
                    교육 {work.education}
                  </span>
                </UiButton>
              ))}
            </div>
          </section>

          {selected && (
          <section
            className="selected-work"
            aria-labelledby="selected-work-title"
          >
            <div className="selected-work__code">
              <small>{selected.code}</small>
              <strong>{selected.place}</strong>
            </div>
            <div className="selected-work__body">
              <p className="overline">SELECTED WORK</p>
              <h2 id="selected-work-title">{selected.title}</h2>
              <dl>
                <div>
                  <dt>시간</dt>
                  <dd>{selected.time}</dd>
                </div>
                <div>
                  <dt>작업팀</dt>
                  <dd>{selected.team}</dd>
                </div>
                <div>
                  <dt>대상화물</dt>
                  <dd>{selected.cargo}</dd>
                </div>
                <div>
                  <dt>교육</dt>
                  <dd>{selected.education}</dd>
                </div>
              </dl>
              <UiButton
                className="text-button"
                onClick={() => setDetailsOpen((open) => !open)}
                type="button"
              >
                {detailsOpen ? '화물 상세 접기' : '화물 상세 보기'}
              </UiButton>
              {detailsOpen && (
                <div className="cargo-detail">
                  <div>
                    <span>01</span>
                    <p>
                      <strong>도료</strong> UN 1263 · Class 3
                    </p>
                  </div>
                  <div>
                    <span>02</span>
                    <p>
                      <strong>세정제</strong> UN 1993 · Class 3
                    </p>
                  </div>
                  <p>MSDS 검수 완료 · 보호구 조건 확정 v3</p>
                </div>
              )}
            </div>
          </section>
          )}

          {selected && (
          <section className="start-section">
            <p>교육 적격성을 다시 확인한 뒤 준비 절차가 열립니다.</p>
            <UiButton
              disabled={selected.education !== '충족'}
              onClick={() => setActiveView('preparation')}
              type="button"
            >
              {selected.education === '충족'
                ? '이 작업 준비 시작'
                : '교육 확인 후 시작 가능'}
            </UiButton>
          </section>
          )}
        </main>

        {selected && activeView === 'preparation' && <PreparationScreen work={selected} />}
        {activeView === 'alerts' && (
          <AlertsScreen onBack={() => setActiveView('today')} />
        )}
        {activeView === 'history' && <HistoryScreen />}
        {activeView === 'profile' && <ProfileScreen />}

        </div>

        <nav aria-label="작업자 메뉴" className="worker-nav">
          <UiButton
            className={activeView === 'today' ? 'is-current' : ''}
            onClick={() => setActiveView('today')}
            type="button"
          >
            <span>오늘 작업</span>
          </UiButton>
          <UiButton
            className={activeView === 'preparation' ? 'is-current' : ''}
            onClick={() => setActiveView('preparation')}
            type="button"
          >
            <span>준비 절차</span>
          </UiButton>
          <UiButton
            className={activeView === 'history' ? 'is-current' : ''}
            onClick={() => setActiveView('history')}
            type="button"
          >
            <span>통과 기록</span>
          </UiButton>
          <UiButton
            className={activeView === 'profile' ? 'is-current' : ''}
            onClick={() => setActiveView('profile')}
            type="button"
          >
            <span>내 정보</span>
          </UiButton>
        </nav>

        {weatherAlertOpen && (
          <div className="weather-alert-layer" role="presentation">
            <section
              aria-labelledby="weather-alert-title"
              aria-modal="true"
              className="weather-alert-sheet"
              role="dialog"
            >
              <header>
                <span>기상 주의 · 오늘 14:00–18:00</span>
                <UiButton
                  aria-label="기상 알림 닫기"
                  onClick={() => setWeatherAlertOpen(false)}
                  type="button"
                >
                  닫기
                </UiButton>
              </header>
              <div className="weather-alert-sheet__body">
                <p className="overline">현재 작업 영향 알림</p>
                <h2 id="weather-alert-title">오후 강풍 예보</h2>
                <p>
                  14시 이후 순간풍속이 증가할 수 있습니다. CFS·야드 작업자는
                  적치물과 이동식 장비의 고정 상태를 작업 전에 다시 확인하세요.
                </p>
                <dl>
                  <div>
                    <dt>영향 작업</dt>
                    <dd>CFS 적출·분류</dd>
                  </div>
                  <div>
                    <dt>작업 구역</dt>
                    <dd>CFS B-3</dd>
                  </div>
                  <div>
                    <dt>확인 기준</dt>
                    <dd>08:55 기상 연동</dd>
                  </div>
                </dl>
              </div>
              <footer>
                <UiButton
                  onClick={() => setWeatherAlertOpen(false)}
                  type="button"
                >
                  지금 닫기
                </UiButton>
                <UiButton
                  onClick={() => {
                    setWeatherAlertOpen(false)
                    setActiveView('alerts')
                  }}
                  type="button"
                >
                  대응 내용 확인
                </UiButton>
              </footer>
            </section>
          </div>
        )}
      </Box>
    </Grid>
  )
}

type PreparationStep = 'education' | 'instruction' | 'ppe' | 'gate'

const PREPARATION_STEPS: {
  id: PreparationStep
  label: string
  state: string
}[] = [
  { id: 'education', label: '안전교육 적격성', state: '충족' },
  { id: 'instruction', label: '당일 안전지침', state: '확인 전' },
  { id: 'ppe', label: '필수 보호구', state: '1 / 3 확인' },
  { id: 'gate', label: '게이트 준비', state: '대기' },
]

function PreparationScreen({ work }: { work: Work }) {
  const [step, setStep] = useState<PreparationStep>('education')
  const [instructionRead, setInstructionRead] = useState(false)
  const [scanned, setScanned] = useState<string[]>([])
  const [educationOpen, setEducationOpen] = useState(false)
  const [educationPlaying, setEducationPlaying] = useState(false)
  const [exceptionOpen, setExceptionOpen] = useState(false)
  const [exceptionReason, setExceptionReason] = useState('')
  const [exceptionMemo, setExceptionMemo] = useState('')
  const [exceptionSubmitted, setExceptionSubmitted] = useState(false)
  const [gateReady, setGateReady] = useState(false)
  const [scanningItem, setScanningItem] = useState<string | null>(null)
  const [scanError, setScanError] = useState('')
  const [lastScan, setLastScan] = useState<TagResult | null>(null)

  const isStepAvailable = (target: PreparationStep) => {
    if (target === 'education') return true
    if (target === 'instruction') return work.education === '충족'
    if (target === 'ppe') return instructionRead
    return instructionRead && scanned.length === PPE_REQUIREMENTS.length
  }

  const scan = async (item: string) => {
    setScanningItem(item)
    setScanError('')

    // NFC 세션이 실패·취소 후에도 종결되지 않는 경우를 대비한 타임아웃 가드:
    // 30초 내 종결 없으면 강제로 상태를 리셋해 재시도 가능하게 한다.
    const timeoutGuard = window.setTimeout(() => {
      setScanningItem(null)
      setScanError('NFC 응답이 없습니다. 버튼을 눌러 다시 시도해 주세요.')
    }, 30000)
    window.clearTimeout(timeoutGuard as unknown as number)

    try {
      const payload = await scanEquipmentTag({
        assignmentId: work.assignmentId,
        equipmentCategory: PPE_REQUIREMENTS.find((r) => r.category === item)?.name ?? item,
      })
      setLastScan(payload)
      // 서버 판정 기준: category가 satisfied면 완료 처리
      if (payload.reasonCode === 'TAG_ACCEPTED' || payload.reasonCode === 'TAG_ALREADY_ACCEPTED') {
        setScanned((items) => (items.includes(item) ? items : [...items, item]))
      } else {
        // 서버 거부(TAG_NOT_REGISTERED 등): 즉시 리셋해 재시도 가능하게
        setScanError(REASON_MESSAGES[payload.reasonCode] ?? '태깅이 거부되었습니다. 다시 시도해 주세요.')
      }
    } catch (error) {
      setScanError(
        error instanceof Error
          ? error.message
          : 'NFC 태그를 확인하지 못했습니다.',
      )
    } finally {
      // 성공·실패 무관 즉시 리셋 — 실패 후 곧바로 재시도할 수 있게 한다.
      window.clearTimeout(timeoutGuard as unknown as number)
      setScanningItem(null)
    }
  }

  return (
    <main className="preparation-screen">
      <header className="flow-header">
        <p className="overline">{work.code}</p>
        <h1>작업 전 준비</h1>
        <p>
          {work.place} · {work.title}
        </p>
      </header>

      <ol aria-label="준비 단계" className="flow-index">
        {PREPARATION_STEPS.map((item, index) => {
          const available = isStepAvailable(item.id)
          return (
            <li
              className={`${step === item.id ? 'is-current' : ''}${available ? '' : ' is-locked'}`}
              key={item.id}
            >
              <UiButton
                disabled={!available}
                onClick={() => setStep(item.id)}
                type="button"
              >
                <span>{String(index + 1).padStart(2, '0')}</span>
                <strong>{item.label}</strong>
                <small>
                  {item.id === 'instruction' && instructionRead
                    ? '확인 완료'
                    : item.id === 'ppe'
                      ? `${lastScan?.progress.satisfied ?? scanned.length} / ${lastScan?.progress.required ?? PPE_REQUIREMENTS.length} 확인`
                      : item.id === 'gate' && gateReady
                        ? '준비 완료'
                        : available
                          ? item.state
                          : '이전 단계 필요'}
                </small>
              </UiButton>
            </li>
          )
        })}
      </ol>

      <section className="step-panel">
        {step === 'education' && (
          <>
            <StepHeading
              number="01"
              title="안전교육 적격성"
              description="법정교육을 충족한 작업자만 다음 준비 단계로 이동할 수 있습니다."
            />
            <div className="eligibility-result">
              <p>판정</p>
              <strong>이 작업에 투입 가능</strong>
              <span>2026.08.23 06:12 자동 판정</span>
            </div>
            <div className="plain-table">
              <div>
                <span>위험물 취급 특별교육</span>
                <strong>16시간 충족</strong>
              </div>
              <div>
                <span>MSDS 교육 · 대상물질</span>
                <strong>PAINT / SOLVENT</strong>
              </div>
              <div>
                <span>기초 안전보건교육</span>
                <strong>정기교육 충족</strong>
              </div>
            </div>
            <UiButton
              className="inline-disclosure"
              onClick={() => setEducationOpen((open) => !open)}
              type="button"
            >
              {educationOpen ? '교육 상세 접기' : '교육 이력·내용 확인'}
            </UiButton>
            {educationOpen && (
              <section
                className="education-detail"
                aria-label="위험물 취급 특별교육 상세"
              >
                <header>
                  <div>
                    <span>필수교육 01</span>
                    <strong>위험물 취급 특별교육</strong>
                  </div>
                  <small>유효 · 2028.05.19까지</small>
                </header>
                <div
                  className={
                    educationPlaying
                      ? 'education-player is-playing'
                      : 'education-player'
                  }
                >
                  <p>교육 영상 · 25분</p>
                  <strong>
                    {educationPlaying
                      ? '재생 중 · 12:34 / 25:00'
                      : 'MSDS와 개인보호구 기본 절차'}
                  </strong>
                  <UiButton
                    onClick={() => setEducationPlaying((playing) => !playing)}
                    type="button"
                  >
                    {educationPlaying ? '일시정지' : '재생 시연'}
                  </UiButton>
                </div>
                <dl>
                  <div>
                    <dt>이수일</dt>
                    <dd>2026.05.20</dd>
                  </div>
                  <div>
                    <dt>수료 조건</dt>
                    <dd>영상 100% · 평가 80점</dd>
                  </div>
                  <div>
                    <dt>이수 결과</dt>
                    <dd>92점 · 수료</dd>
                  </div>
                </dl>
              </section>
            )}
            <UiButton
              className="panel-action"
              onClick={() => setStep('instruction')}
              type="button"
            >
              다음 · 안전지침 확인
            </UiButton>
          </>
        )}

        {step === 'instruction' && (
          <>
            <StepHeading
              number="02"
              title="당일 안전지침"
              description="현재 작업과 화물에 적용되는 최신 지침입니다."
            />
            <div className="document-meta">
              <span>SI-260823-CFS-03 · VERSION 4</span>
              <time>08.23 05:40 확정</time>
            </div>
            <article className="instruction-copy">
              <h2>혼재 인화성 액체 취급 지침</h2>
              <h3>작업 전</h3>
              <p>
                용기 밀폐 상태와 누출 흔적을 확인하고, 점화원이 있는 장비는
                작업구역 밖으로 이동합니다.
              </p>
              <h3>작업 중</h3>
              <p>
                환기 상태를 유지하고 정전기 발생 가능성이 있는 도구를 사용하지
                않습니다. 이상 냄새나 누출을 발견하면 즉시 작업을 중지합니다.
              </p>
              <h3>기상 주의</h3>
              <p>
                14시 이후 강풍이 예상됩니다. 적치물과 이동식 장비의 고정 상태를
                재확인합니다.
              </p>
            </article>
            <label className="acknowledge-row">
              <UiInput
                checked={instructionRead}
                onChange={(event) => setInstructionRead(event.target.checked)}
                type="checkbox"
              />
              <span>지침 전체 내용을 읽고 작업 시 준수하겠습니다.</span>
            </label>
            <UiButton
              className="panel-action"
              disabled={!instructionRead}
              onClick={() => setStep('ppe')}
              type="button"
            >
              확인 기록 후 보호구 준비
            </UiButton>
          </>
        )}

        {step === 'ppe' && (
          <>
            <StepHeading
              number="03"
              title="필수 보호구"
              description="MSDS 8항과 작업유형을 기준으로 확정된 목록입니다."
            />
            <div className="ppe-list">
              {PPE_REQUIREMENTS.map((req, index) => {
                const complete = scanned.includes(req.category)
                return (
                  <div key={req.category} className="ppe-row">
                    <span>{String(index + 1).padStart(2, '0')}</span>
                    <div>
                      <strong>{req.name}</strong>
                      <small>{req.description}</small>
                    </div>
                    <UiButton
                      disabled={complete || scanningItem !== null}
                      onClick={() => void scan(req.category)}
                      type="button"
                    >
                      {complete
                        ? '확인됨'
                        : scanningItem === req.category
                          ? 'NFC 읽는 중…'
                          : 'NFC 읽기'}
                    </UiButton>
                  </div>
                )
              })}
            </div>
            <div className="nfc-note">
              <strong>NFC 입력 규칙</strong>
              <p>
                태그 토큰만 전송하며 장비 종류·소유자·사용중지 여부는 서버에서
                판정합니다.
              </p>
            </div>
            {lastScan && (
              <section className="nfc-receipt" role="status">
                <header>
                  <strong>NFC 확인 완료</strong>
                  <time>방금 전</time>
                </header>
                <dl>
                  <div>
                    <dt>보호구</dt>
                    <dd>{lastScan.payload.equipmentCategory}</dd>
                  </div>
                  <div>
                    <dt>입력 경로</dt>
                    <dd>
                      {lastScan.payload.device.channel === 'phone-nfc'
                        ? '휴대폰 NFC'
                        : '시연용 태그'}
                    </dd>
                  </div>
                  <div>
                    <dt>판정 사유</dt>
                    <dd className="nfc-receipt__token">
                      {lastScan.reasonCode}
                    </dd>
                  </div>
                </dl>
              </section>
            )}
            {scanError && (
              <p className="nfc-scan-error" role="alert">
                {scanError}
              </p>
            )}
            <UiButton
              className="exception-toggle"
              onClick={() => setExceptionOpen((open) => !open)}
              type="button"
            >
              보호구 문제 신고·예외 승인 요청
            </UiButton>
            {exceptionOpen && (
              <section aria-label="예외 승인 요청" className="exception-form">
                {exceptionSubmitted ? (
                  <div className="exception-result">
                    <span>승인 대기</span>
                    <strong>
                      관리자 검토 전까지 게이트 통과가 제한됩니다.
                    </strong>
                    <p>
                      요청번호 EX-260823-014 · 대체 보호구 또는 현장 조치가
                      확인되어야 합니다.
                    </p>
                  </div>
                ) : (
                  <>
                    <p>
                      예외 승인은 교육 미이수나 보호구 미착용을 면제하지
                      않습니다. 장비 교체·대체품 확인이 필요한 경우에만
                      요청하세요.
                    </p>
                    <label>
                      <span>요청 사유</span>
                      <select
                        value={exceptionReason}
                        onChange={(event) =>
                          setExceptionReason(event.target.value)
                        }
                      >
                        <option value="">선택해 주세요</option>
                        <option value="damaged">태그 장비 파손·사용중지</option>
                        <option value="replacement">
                          대체 보호구 확인 필요
                        </option>
                        <option value="personal">개인 보호구 등록 요청</option>
                      </select>
                    </label>
                    <label>
                      <span>현장 메모</span>
                      <textarea
                        maxLength={300}
                        onChange={(event) =>
                          setExceptionMemo(event.target.value)
                        }
                        placeholder="장비명과 현재 상황을 입력하세요."
                        value={exceptionMemo}
                      />
                    </label>
                    <UiButton
                      disabled={
                        !exceptionReason || exceptionMemo.trim().length < 5
                      }
                      onClick={() => setExceptionSubmitted(true)}
                      type="button"
                    >
                      관리자에게 검토 요청
                    </UiButton>
                  </>
                )}
              </section>
            )}
            <UiButton
              className="panel-action"
              disabled={scanned.length < PPE_REQUIREMENTS.length}
              onClick={() => setStep('gate')}
              type="button"
            >
              게이트 준비상태 확인
            </UiButton>
          </>
        )}

        {step === 'gate' && (
          <>
            <StepHeading
              number="04"
              title="게이트 준비"
              description="사원증 태깅 시 아래 조건을 서버에서 다시 검증합니다."
            />
            <div className="gate-checks">
              <div>
                <span>작업 배정</span>
                <strong>유효</strong>
              </div>
              <div>
                <span>안전교육</span>
                <strong>충족</strong>
              </div>
              <div>
                <span>안전지침</span>
                <strong>{instructionRead ? '확인 완료' : '확인 필요'}</strong>
              </div>
              <div>
                <span>필수 보호구</span>
                <strong>
                  {scanned.length === PPE_REQUIREMENTS.length
                    ? '3종 완료'
                    : `${PPE_REQUIREMENTS.length - scanned.length}종 미확인`}
                </strong>
              </div>
              <div>
                <span>작업중지</span>
                <strong>없음</strong>
              </div>
            </div>
            {gateReady ? (
              <div className="gate-ready-result">
                <span>READY · 07:36:18</span>
                <strong>게이트 이동 가능</strong>
                <p>
                  1부두 정문 태블릿에서 사원증을 태그하면 서버가 모든 조건을
                  다시 판정합니다.
                </p>
                <dl>
                  <div>
                    <dt>작업</dt>
                    <dd>{work.code}</dd>
                  </div>
                  <div>
                    <dt>준비기록</dt>
                    <dd>PREP-260823-0031</dd>
                  </div>
                </dl>
              </div>
            ) : (
              <div className="gate-direction">
                <span>GATE</span>
                <strong>1부두 정문</strong>
                <p>
                  준비 완료를 기록한 뒤 게이트 리더에 사원증을 태그해 주세요.
                </p>
              </div>
            )}
            <UiButton
              className="panel-action"
              disabled={!instructionRead || scanned.length < PPE_REQUIREMENTS.length || gateReady}
              onClick={() => setGateReady(true)}
              type="button"
            >
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
    {
      id: 'wind',
      level: '주의',
      time: '오늘 07:10',
      title: '오후 강풍 예보',
      body: '14시 이후 순간풍속 증가가 예상됩니다. CFS·야드 작업은 적치물과 이동식 장비 고정을 재확인합니다.',
      source: '기상 연동',
    },
    {
      id: 'crane',
      level: '긴급',
      time: '오늘 08:40',
      title: '3부두 크레인 점검 중',
      body: '점검 종료 공지 전까지 3부두 작업자 출입과 장비 접근을 금지합니다.',
      source: '안전관리자',
    },
    {
      id: 'wave',
      level: '주의',
      time: '어제 14:20',
      title: '높은 파고 주의',
      body: '해상 작업자는 구명조끼 체결 상태와 작업구역 통제선을 확인하세요.',
      source: '기상 연동',
    },
  ]

  return (
    <main className="alerts-screen">
      <header>
        <UiButton onClick={onBack} type="button">
          ← 오늘 작업
        </UiButton>
        <p className="overline">BUSAN PORT · LIVE</p>
        <h1>기상·긴급 알림</h1>
        <p>현재 작업에 영향을 주는 공지만 우선 표시합니다.</p>
      </header>
      <section className="weather-summary">
        <div>
          <span>현재</span>
          <strong>27°C</strong>
        </div>
        <dl>
          <div>
            <dt>풍속</dt>
            <dd>3m/s</dd>
          </div>
          <div>
            <dt>습도</dt>
            <dd>62%</dd>
          </div>
          <div>
            <dt>기준</dt>
            <dd>08:55</dd>
          </div>
        </dl>
      </section>
      <section aria-label="안전 알림 목록" className="notice-list">
        {notices.map((notice) => {
          const done = acknowledged.includes(notice.id)
          return (
            <article
              className={notice.level === '긴급' ? 'is-urgent' : ''}
              key={notice.id}
            >
              <header>
                <span>{notice.level}</span>
                <time>{notice.time}</time>
              </header>
              <h2>{notice.title}</h2>
              <p>{notice.body}</p>
              <footer>
                <small>{notice.source}</small>
                <UiButton
                  disabled={done}
                  onClick={() =>
                    setAcknowledged((items) => [...items, notice.id])
                  }
                  type="button"
                >
                  {done ? '확인 기록됨' : '내용 확인'}
                </UiButton>
              </footer>
            </article>
          )
        })}
      </section>
    </main>
  )
}

function StepHeading({
  number,
  title,
  description,
}: {
  number: string
  title: string
  description: string
}) {
  return (
    <header className="step-heading">
      <span>{number}</span>
      <div>
        <h2>{title}</h2>
        <p>{description}</p>
      </div>
    </header>
  )
}

function HistoryScreen() {
  return (
    <main className="simple-screen">
      <header>
        <p className="overline">LAST 90 DAYS</p>
        <h1>게이트 통과 기록</h1>
        <p>통과와 차단 결과, 판정 사유를 확인합니다.</p>
      </header>
      <div className="history-list">
        <div>
          <time>
            08.23
            <br />
            07:42
          </time>
          <p>
            <strong>1부두 정문 · 통과</strong>
            <span>WB-260823-03 · 정기 출입</span>
          </p>
          <small>PASS</small>
        </div>
        <div>
          <time>
            08.22
            <br />
            08:05
          </time>
          <p>
            <strong>1부두 정문 · 통과</strong>
            <span>WB-260822-03 · 정기 출입</span>
          </p>
          <small>PASS</small>
        </div>
        <div>
          <time>
            08.21
            <br />
            07:51
          </time>
          <p>
            <strong>1부두 정문 · 차단</strong>
            <span>필수 보호구 1종 미확인</span>
          </p>
          <small>BLOCK</small>
        </div>
      </div>
    </main>
  )
}

function ProfileScreen() {
  return (
    <main className="simple-screen">
      <header>
        <p className="overline">EMP-240031</p>
        <h1>윤서진</h1>
        <p>CFS운영팀 · 화물 분류 작업자</p>
      </header>
      <dl className="profile-list">
        <div>
          <dt>계정 상태</dt>
          <dd>재직 · 사용 가능</dd>
        </div>
        <div>
          <dt>소속 팀</dt>
          <dd>CFS 2조</dd>
        </div>
        <div>
          <dt>등록 사원증</dt>
          <dd>•••• 8A21</dd>
        </div>
        <div>
          <dt>선호 언어</dt>
          <dd>한국어</dd>
        </div>
      </dl>
      <UiButton className="secondary-action" type="button">
        비밀번호 변경
      </UiButton>
    </main>
  )
}
