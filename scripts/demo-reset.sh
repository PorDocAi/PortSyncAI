#!/usr/bin/env bash
# ============================================================
# PortSyncAI 데모 리셋 스크립트
# 사용법: ./demo-reset.sh [백엔드경로]
# 기본: /Users/wingwogus/Projects/PortDocAI-front-integration
# ============================================================
set -e

ROOT="${1:-/Users/wingwogus/Projects/PortDocAI-front-integration}"
DB="/tmp/e2e-flow.db"
PORT=18090
BASE="http://127.0.0.1:$PORT"

echo "════════ PortSyncAI 데모 리셋 시작 ════════"

# ── 1) 백엔드 종료 (돌고 있으면) ──
if lsof -t -i :$PORT >/dev/null 2>&1; then
  echo "[1] 백엔드 종료..."
  kill $(lsof -t -i :$PORT) 2>/dev/null || true
  sleep 2
fi

# ── 2) DB 클린 리셋 ──
echo "[2] DB 리셋 (배정·태깅·게이트 기록 초기화)"
sqlite3 "$DB" <<'SQL'
-- 태깅·할당·게이트 기록 제거
DELETE FROM work_assignment_equipment;
DELETE FROM equipment_check_events;
DELETE FROM equipment_check_logs;
DELETE FROM shared_equipment_claims;
DELETE FROM gate_verify_logs;
DELETE FROM gate_events;

-- 홍길동(employee 2) 출근 상태 초기화
UPDATE attendances SET gate_status='BLOCKED', gate_passed_at=NULL,
  equipment_check_completed=0, instruction_ack_completed=0, approval_status='NOT_REQUIRED'
WHERE employee_id=2;

-- 배정: WA-5만 남기고 SELECTED로 복원
DELETE FROM v2_work_assignments WHERE employee_id=2 AND work_assignment_id != 5;
UPDATE v2_work_assignments SET status='SELECTED', started_at=NULL, completed_at=NULL WHERE work_assignment_id=5;
SQL
echo "    → WA-5 클린 상태 (4종 PPE 요구, 미태깅)"

# ── 3) 백엔드 기동 (새 터미널 세션 필요 없이 nohup) ──
echo "[3] 백엔드 기동 ($PORT)"
cd "$ROOT/apis/api"
PORT=$PORT DATABASE_URL="sqlite:$DB?mode=rwc" JWT_SECRET='e2e-secret-key' UPLOAD_DIR=/tmp/e2e-uploads \
  nohup cargo run > /tmp/api.log 2>&1 &
disown

for i in $(seq 1 40); do
  curl -fsS "$BASE/health" >/dev/null 2>&1 && { echo "    → READY"; break; }
  sleep 2
done

# ── 4) 로그인 + 지침 확인 (자동화 가능한 부분 선처리) ──
TOKEN=$(curl -sS -X POST $BASE/auth/signin -H 'Content-Type: application/json' \
  -d '{"email":"worker@port.test","password":"worker1234"}' | python3 -c 'import sys,json;print(json.load(sys.stdin)["token"])')
curl -sS -X POST $BASE/attendances -H "Authorization: Bearer $TOKEN" >/dev/null
for IID in 1001 1003; do
  curl -sS -X POST $BASE/attendances/ack -H "Authorization: Bearer $TOKEN" \
    -H 'Content-Type: application/json' -d "{\"instruction_id\":$IID,\"scrolled_to_end\":true}" >/dev/null
done
echo "[4] 로그인·지침 확인 완료 (앱에서도 로그인되어 있으면 그대로 사용)"

# ── 5) 상태 요약 ──
echo ""
echo "════════ 리셋 완료 — 앱에서 진행할 순서 ════════"
echo "  HyunPhone(worker):"
echo "    ① 준비 화면 → 보호구 4종 NFC 태깅"
echo "       안전화=demo-boot-token-003  장갑=demo-glove-token-001"
echo "       헬멧=demo-helmet-token-002  작업복=demo-uniform-token-005"
echo "    ② 4종 완료 시 자동으로 준비완료 처리됨"
echo "  iPhone SE(gate-terminal):"
echo "    ③ GATE 버튼 → 단말 토큰 등록(최초 1회):"
echo "       838c0a29-591c-4b78-98ac-1500e5b748e4"
echo "    ④ 사원증 NFC 태깅 (UID: EMP-WORKER-0002-UID) → PASS"
echo ""
echo "  ※ 참고: 장비 완료 선언(④ 이전)은 서버가 태깅 4종을 감지해 자동 처리함"
echo "══════════════════════════════════════════════"
