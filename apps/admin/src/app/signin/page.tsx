import { Button as UiButton, Input as UiInput } from '@devup-ui/react'
import Link from 'next/link'

export default function SignInPage() {
  return (
    <main className="admin-signin">
      <section>
        <div className="admin-wordmark admin-wordmark--large"><span>PS</span><strong>PortSyncAI</strong></div>
        <div className="admin-signin__title">
          <p className="admin-overline">ADMINISTRATOR ACCESS</p>
          <h1>관리자 로그인</h1>
          <p>권한이 부여된 사내 계정으로 접속합니다.</p>
        </div>
        <form>
          <label>사내 이메일<UiInput autoComplete="username" placeholder="name@company.com" type="email" /></label>
          <label>비밀번호<UiInput autoComplete="current-password" placeholder="비밀번호 입력" type="password" /></label>
          <Link href="/dashboard">로그인</Link>
        </form>
        <footer><span>접속 기록과 관리 작업은 감사로그에 저장됩니다.</span><UiButton type="button">비밀번호 초기화</UiButton></footer>
      </section>
    </main>
  )
}
