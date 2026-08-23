'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'


import Link from 'next/link'
import { useEffect, useState } from 'react'

type GateState = 'idle' | 'checking' | 'pass' | 'block'

const PASS_CHECKS = [
  ['작업 배정', 'WB-260823-03 · 유효'],
  ['안전교육', '요구 교육 충족'],
  ['안전지침', 'VERSION 4 확인'],
  ['보호구 준비', '필수 항목 3 / 3'],
  ['작업중지', '발령 없음'],
]

export default function GateTerminalPage() {
  const [state, setState] = useState<GateState>('idle')
  const [credential, setCredential] = useState('04A8-1F29-77C2')
  const [blockedReason, setBlockedReason] = useState('EDUCATION_MISSING')
  const [now, setNow] = useState(() => new Date())
  const [resetIn, setResetIn] = useState(15)

  useEffect(() => {
    const timer = window.setInterval(() => setNow(new Date()), 1000)
    return () => window.clearInterval(timer)
  }, [])

  useEffect(() => {
    if (state !== 'pass' && state !== 'block') return
    setResetIn(15)
    const timer = window.setInterval(() => {
      setResetIn((seconds) => {
        if (seconds <= 1) {
          window.clearInterval(timer)
          setState('idle')
          return 15
        }
        return seconds - 1
      })
    }, 1000)
    return () => window.clearInterval(timer)
  }, [state])

  const evaluate = (result: 'pass' | 'block') => {
    setState('checking')
    window.setTimeout(() => setState(result), 650)
  }

  const toggleFullscreen = async () => {
    if (document.fullscreenElement) await document.exitFullscreen()
    else await document.documentElement.requestFullscreen()
  }

  const timestamp = new Intl.DateTimeFormat('ko-KR', {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  }).format(now)

  return (
    <main className={`gate-terminal gate-terminal--${state}`}>
      <header className="gate-terminal__header">
        <div><span className="gate-terminal__mark">PS</span><strong>PortSyncAI GATE</strong></div>
        <dl><div><dt>단말</dt><dd>GATE-01 / 1부두 정문</dd></div><div><dt>리더·API</dt><dd><i aria-hidden="true" /> 준비 완료</dd></div><div><dt>시각</dt><dd>{timestamp}</dd></div></dl>
        <div className="gate-terminal__actions"><UiButton onClick={toggleFullscreen} type="button">전체 화면</UiButton><Link href="/dashboard">관리</Link></div>
      </header>

      {state === 'idle' && (
        <section className="gate-idle">
          <p className="gate-kicker">ACCESS VERIFICATION</p>
          <h1>사원증을<br />태그해 주세요</h1>
          <p>작업 배정과 교육, 지침, 보호구 준비 상태를 확인합니다.</p>
          <form onSubmit={(event) => { event.preventDefault(); evaluate('pass') }}>
            <label>리더 입력 버퍼<UiInput aria-label="사원증 UID" autoFocus onChange={(event) => setCredential(event.target.value)} value={credential} /></label>
            <UiButton type="submit">입력값 판정</UiButton>
          </form>
          <details className="gate-demo-controls"><summary>태블릿 시연 도구</summary><div><UiButton onClick={() => evaluate('pass')} type="button">PASS 시연</UiButton><UiButton onClick={() => evaluate('block')} type="button">BLOCK 시연</UiButton></div></details>
          <footer><span>NFC READER · HID KEYBOARD MODE</span><span>태그 UID 원문은 감사로그에 마스킹 저장</span></footer>
        </section>
      )}

      {state === 'checking' && (
        <section className="gate-checking" aria-live="polite">
          <p className="gate-kicker">VERIFYING · {credential}</p>
          <h1>출입 조건을<br />확인하고 있습니다</h1>
          <div><span>01 작업자 식별</span><span>02 교육 적격성</span><span>03 작업 준비</span><span>04 작업중지</span></div>
        </section>
      )}

      {state === 'pass' && (
        <section className="gate-decision" aria-live="assertive">
          <div className="gate-decision__word"><p>홍길동 · EMP-240031</p><h1>PASS</h1><strong>통과</strong><span>10:20:19 · EVENT GE-260823-092</span></div>
          <div className="gate-decision__detail"><header><span className="gate-kicker">WB-260823-03</span><h2>CFS B-3</h2><p>CFS 적출·분류 · CFS 2조</p></header><dl>{PASS_CHECKS.map(([term, value]) => <div key={term}><dt>{term}</dt><dd>{value}</dd></div>)}</dl><p className="gate-auto-reset">{resetIn}초 후 자동으로 대기 화면으로 돌아갑니다.</p><UiButton onClick={() => setState('idle')} type="button">다음 작업자 대기</UiButton></div>
        </section>
      )}

      {state === 'block' && (
        <section className="gate-decision gate-decision--block" aria-live="assertive">
          <div className="gate-decision__word"><p>김태완 · EMP-240044</p><h1>BLOCK</h1><strong>출입 차단</strong><span>10:20:19 · EVENT GE-260823-092</span></div>
          <div className="gate-decision__detail"><header><span className="gate-kicker">WB-260823-05</span><h2>교육 미충족</h2><p>필수 조건을 충족하기 전에는 출입할 수 없습니다.</p></header><dl><div><dt>차단 코드</dt><dd>{blockedReason}</dd></div><div><dt>미충족 항목</dt><dd>MSDS 교육 · UN 1263</dd></div><div><dt>다음 조치</dt><dd>안전교육 담당자 확인</dd></div><div><dt>판정 원칙</dt><dd>FAIL CLOSED</dd></div></dl><label>시연 차단 사유<select onChange={(event) => setBlockedReason(event.target.value)} value={blockedReason}><option>EDUCATION_MISSING</option><option>PPE_INCOMPLETE</option><option>WORK_STOPPED</option><option>SERVICE_UNAVAILABLE</option></select></label><p className="gate-auto-reset">{resetIn}초 후 자동으로 대기 화면으로 돌아갑니다.</p><UiButton onClick={() => setState('idle')} type="button">확인 후 대기 화면</UiButton></div>
        </section>
      )}
    </main>
  )
}
