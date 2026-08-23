'use client'

import { Input as UiInput } from '@devup-ui/react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useState } from 'react'

import { BrandLockup } from '@/components/BrandLockup'
import { apiClient } from '@/lib/apiClient'

export default function SignInPage() {
  const router = useRouter()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      const res = await apiClient.signin({ email, password })
      localStorage.setItem('ps_auth', JSON.stringify(res))
      localStorage.setItem('ps_token', res.token)
      router.push('/')
    } catch (err) {
      setError(
        err instanceof Error && err.message.includes('401')
          ? '이메일 또는 비밀번호가 올바르지 않습니다.'
          : '로그인에 실패했습니다. 잠시 후 다시 시도해 주세요.',
      )
    } finally {
      setLoading(false)
    }
  }

  return (
    <main className="signin-page">
      <section aria-labelledby="signin-title" className="signin-panel">
        <BrandLockup className="wordmark--large" />
        <div className="signin-copy">
          <p className="overline">WORKER ACCESS</p>
          <h1 id="signin-title">작업자 로그인</h1>
          <p>회사에서 발급한 계정으로 로그인해 주세요.</p>
        </div>
        <form className="signin-form" onSubmit={handleSubmit}>
          <label>
            이메일
            <UiInput
              autoComplete="username"
              name="email"
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEmail(e.target.value)}
              placeholder="예: worker@port.test"
              required
              type="email"
              value={email}
            />
          </label>
          <label>
            비밀번호
            <UiInput
              autoComplete="current-password"
              name="password"
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)}
              placeholder="비밀번호 입력"
              required
              type="password"
              value={password}
            />
          </label>
          {error && (
            <p role="alert" style={{ color: '#e53e3e', fontSize: 13 }}>
              {error}
            </p>
          )}
          <button
            type="submit"
            className="primary-link"
            disabled={loading}
            style={{ cursor: loading ? 'wait' : 'pointer' }}
          >
            {loading ? '로그인 중…' : '로그인'}
          </button>
        </form>
        <p className="signin-help">
          계정 발급·초기화는 현장 관리자에게 문의하세요.
        </p>
      </section>
    </main>
  )
}
