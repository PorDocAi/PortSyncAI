import { Input as UiInput } from '@devup-ui/react'
import Link from 'next/link'
import { BrandLockup } from '@/components/BrandLockup'

export default function SignInPage() {
  return (
    <main className="signin-page">
      <section className="signin-panel" aria-labelledby="signin-title">
        <BrandLockup className="wordmark--large" />
        <div className="signin-copy">
          <p className="overline">WORKER ACCESS</p>
          <h1 id="signin-title">작업자 로그인</h1>
          <p>회사에서 발급한 계정으로 로그인해 주세요.</p>
        </div>
        <form className="signin-form">
          <label>
            사번
            <UiInput name="employeeNumber" placeholder="예: EMP-240031" autoComplete="username" />
          </label>
          <label>
            비밀번호
            <UiInput name="password" type="password" placeholder="비밀번호 입력" autoComplete="current-password" />
          </label>
          <Link className="primary-link" href="/">로그인</Link>
        </form>
        <p className="signin-help">계정 발급·초기화는 현장 관리자에게 문의하세요.</p>
      </section>
    </main>
  )
}
