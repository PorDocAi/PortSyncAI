import { WorkerWorkspace } from '@/components/worker/WorkerWorkspace'
import GateTerminalPage from '@/app/gate-terminal/page'

export default function HomePage() {
  // 게이트 단말 전용 빌드(GATE_TERMINAL=true)는 앱 실행 즉시 게이트 화면을 띄운다.
  if (process.env.GATE_TERMINAL === 'true') {
    return <GateTerminalPage />
  }
  return <WorkerWorkspace />
}
