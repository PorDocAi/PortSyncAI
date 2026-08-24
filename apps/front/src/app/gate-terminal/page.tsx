'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'

import { apiClient } from '@/lib/apiClient'

import { useEffect, useState } from 'react'
import { BrandLockup } from '@/components/BrandLockup'

type GateState = 'idle' | 'checking' | 'pass' | 'block'

type VerifyResponse = {
  allowed: boolean
  employee_name: string
  reason: string
}

const TERMINAL_TOKEN_KEY = 'ps_gate_terminal_token'

export default function GateTerminalPage() {
  const [state, setState] = useState<GateState>('idle')
  const [credential, setCredential] = useState('')
  const [decision, setDecision] = useState<VerifyResponse | null>(null)
  const [terminalToken, setTerminalToken] = useState('')
  const [tokenInput, setTokenInput] = useState('')
  const [setupError, setSetupError] = useState('')
  const [now, setNow] = useState(() => new Date())
  const [resetIn, setResetIn] = useState(15)

  // 단말 토큰 복원
  useEffect(() => {
    const saved = localStorage.getItem(TERMINAL_TOKEN_KEY)
    if (saved) setTerminalToken(saved)
  }, [])

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
          setDecision(null)
          return 15
        }
        return seconds - 1
      })
    }, 1000)
    return () => window.clearInterval(timer)
  }, [state])

  const saveToken = () => {
    const trimmed = tokenInput.trim()
    if (!trimmed) {
      setSetupError('단말 토큰을 입력해 주세요.')
      return
    }
    localStorage.setItem(TERMINAL_TOKEN_KEY, trimmed)
    setTerminalToken(trimmed)
    setTokenInput('')
    setSetupError('')
  }

  const verify = async () => {
    const uid = credential.trim()
    if (!uid) return
    if (!terminalToken) {
      setSetupError('먼저 게이트 단말 토큰을 등록해 주세요.')
      return
    }
    setState('checking')
    try {
      const res = await fetch(
        `${process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://127.0.0.1:18090'}/gate/verify`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${terminalToken}`,
          },
          body: JSON.stringify({ nfc_card_uid: uid }),
        },
      )
      if (res.status === 404) {
        // 미등록 사원증
        setDecision({
          allowed: false,
          employee_name: '미등록 사원증',
          reason: '등록되지 않은 카드입니다.',
        })
        setState('block')
        setCredential('')
        return
      }
      const body = (await res.json()) as VerifyResponse
      setDecision(body)
      setState(body.allowed ? 'pass' : 'block')
      setCredential('')
    } catch {
      setDecision({
        allowed: false,
        employee_name: '통신 오류',
        reason: '게이트 서버에 연결할 수 없습니다.',
      })
      setState('block')
      setCredential('')
    }
  }

  const onSubmit = (event: React.FormEvent) => {
    event.preventDefault()
    void verify()
  }

  const toggleFullscreen = async () => {
    if (document.fullscreenElement) await document.exitFullscreen()
    else await document.documentElement.requestFullscreen()
  }

  const timestamp = new Intl.DateTimeFormat('ko-KR', {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
  }).format(now)

  if (!terminalToken) {
    return (
      <main className="gate-terminal gate-terminal--idle">
        <header className="gate-terminal__header">
          <BrandLockup />
          
        </header>
        <section className="gate-idle">
          <p className="gate-kicker">단말 등록</p>
          <h1>게이트 단말<br />토큰 입력</h1>
          <p>관리자 화면에서 게이트 단말을 발급하면 평문 토큰이 1회 표시됩니다. 발급된 토큰을 입력해 주세요.</p>
          <form onSubmit={(event) => { event.preventDefault(); saveToken() }}>
            <label>단말 토큰<UiInput aria-label="게이트 단말 토큰" autoFocus onChange={(event) => setTokenInput(event.target.value)} value={tokenInput} /></label>
            {setupError && <p style={{ color: '#e53e3e', fontSize: 13 }}>{setupError}</p>}
            <UiButton type="submit">등록</UiButton>
          </form>
          <footer><span>GATE TERMINAL SETUP</span><span>토큰은 이 기기에만 저장됩니다</span></footer>
        </section>
      </main>
    )
  }

  return (
    <main className={`gate-terminal gate-terminal--${state}`}>
      <header className="gate-terminal__header">
        <BrandLockup />
        <dl><div><dt>단말</dt><dd>GATE / 북문</dd></div><div><dt>리더·API</dt><dd><i aria-hidden="true" /> 준비 완료</dd></div><div><dt>시각</dt><dd>{timestamp}</dd></div></dl>
        <div className="gate-terminal__actions"><UiButton onClick={toggleFullscreen} type="button">전체 화면</UiButton><UiButton onClick={() => { localStorage.removeItem(TERMINAL_TOKEN_KEY); setTerminalToken('') }} type="button">토큰 해제</UiButton></div>
      </header>

      {state === 'idle' && (
        <section className="gate-idle">
          <p className="gate-kicker">게이트 출입 확인</p>
          <h1>사원증을<br />태그해 주세요</h1>
          <p>작업 배정과 교육, 지침, 보호구 준비 상태를 확인합니다.</p>
          <form onSubmit={onSubmit}>
            <label>사원증 UID<UiInput aria-label="사원증 UID" autoFocus onChange={(event) => setCredential(event.target.value)} value={credential} placeholder="EMP-WORKER-0002-UID" /></label>
            <UiButton type="submit">입력값 판정</UiButton>
          </form>
          <footer><span>NFC READER · KEYBOARD ENTRY</span><span>태그 UID 원문은 감사로그에 기록됩니다</span></footer>
        </section>
      )}

      {state === 'checking' && (
        <section className="gate-checking" aria-live="polite">
          <p className="gate-kicker">판정 중 · {credential}</p>
          <h1>출입 조건을<br />확인하고 있습니다</h1>
          <div><span>01 작업자 식별</span><span>02 교육 적격성</span><span>03 작업 준비</span><span>04 작업중지</span></div>
        </section>
      )}

      {state === 'pass' && decision && (
        <section className="gate-decision" aria-live="assertive">
          <div className="gate-decision__word"><p>{decision.employee_name}</p><h1>PASS</h1><strong>통과</strong><span>{timestamp}</span></div>
          <div className="gate-decision__detail"><header><span className="gate-kicker">GATE</span><h2>북문</h2><p>당일 준비 절차가 완료된 작업자입니다.</p></header><dl><div><dt>판정 결과</dt><dd>{decision.reason}</dd></div><div><dt>안전지침</dt><dd>확인 완료</dd></div><div><dt>필수 보호구</dt><dd>착용 확인</dd></div><div><dt>작업중지</dt><dd>발령 없음</dd></div></dl><p className="gate-auto-reset">{resetIn}초 후 자동으로 대기 화면으로 돌아갑니다.</p><UiButton onClick={() => { setState('idle'); setDecision(null) }} type="button">다음 작업자 대기</UiButton></div>
        </section>
      )}

      {state === 'block' && decision && (
        <section className="gate-decision gate-decision--block" aria-live="assertive">
          <div className="gate-decision__word"><p>{decision.employee_name}</p><h1>BLOCK</h1><strong>출입 차단</strong><span>{timestamp}</span></div>
          <div className="gate-decision__detail"><header><span className="gate-kicker">GATE</span><h2>북문</h2><p>필수 조건을 충족하기 전에는 출입할 수 없습니다.</p></header><dl><div><dt>차단 사유</dt><dd>{decision.reason}</dd></div><div><dt>판정 원칙</dt><dd>FAIL CLOSED</dd></div></dl><p className="gate-auto-reset">{resetIn}초 후 자동으로 대기 화면으로 돌아갑니다.</p><UiButton onClick={() => { setState('idle'); setDecision(null) }} type="button">확인 후 대기 화면</UiButton></div>
        </section>
      )}
    </main>
  )
}
