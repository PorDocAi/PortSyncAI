'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'
import { BrandLockup } from '@/components/BrandLockup'
import { apiClient } from '@/lib/apiClient'
import { useRouter } from 'next/navigation'
import { useState } from 'react'

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
      if (res.system_role !== 'ADMIN') {
        setError('관리자 계정으로만 접속할 수 있습니다.')
        return
      }
      localStorage.setItem('ps_admin_auth', JSON.stringify(res))
      router.push('/dashboard')
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
    <main className="admin-signin">
      <section>
        <BrandLockup className="admin-wordmark--large" />
        <div className="admin-signin__title">
          <p className="admin-overline">ADMINISTRATOR ACCESS</p>
          <h1>관리자 로그인</h1>
          <p>권한이 부여된 사내 계정으로 접속합니다.</p>
        </div>
        <form onSubmit={handleSubmit}>
          <label>사내 이메일
            <UiInput autoComplete="username" placeholder="name@company.com" type="email"
              value={email} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setEmail(e.target.value)} required />
          </label>
          <label>비밀번호
            <UiInput autoComplete="current-password" placeholder="비밀번호 입력" type="password"
              value={password} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setPassword(e.target.value)} required />
          </label>
          {error && <p style={{ color: '#e53e3e', fontSize: 13 }} role="alert">{error}</p>}
          <UiButton type="submit">{loading ? '로그인 중…' : '로그인'}</UiButton>
        </form>
        <footer><span>접속 기록과 관리 작업은 감사로그에 저장됩니다.</span></footer>
      </section>
    </main>
  )
}
