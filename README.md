# PortSyncAI

> 화물 문서와 작업 정보를 연결해 작업 전 안전교육, 안전지침, 보호구 준비 상태를
> 확인하고 게이트 통과 여부를 일관되게 판정하는 항만 안전 게이트키핑 플랫폼

**Team Portsync** — Port(항만) × Sync(동기화): 사람, 시스템, 안전을 하나로 연결합니다.

## 왜 필요한가

- 작업 투입 전에 반드시 확인해야 하는 조건이 문서·교육·장비 시스템에 흩어져 있어 누락 여부를 한 번에 확인하기 어렵다.
- 시스템은 교육 적격성을 우선 판정하고, 지침 확인·보호구 준비·작업중지 상태를 합쳐 출입을 허용하거나 차단한다.
- 각 확인과 판정 결과를 감사로그로 남겨 현장 안전관리와 사후 추적을 지원한다.

## 핵심 기능

| 기능 | 설명 |
|---|---|
| 🚪 게이트 판정 | 유효한 작업 배정, 교육, 지침 확인, 보호구 준비, 작업중지 상태를 종합해 PASS/BLOCK 판정 |
| 🎓 안전교육 | 작업과 화물에 필요한 교육의 이수시간·유효성을 먼저 확인하고 미충족 작업자의 투입 차단 |
| 📋 안전지침 | 작업별 최신 지침과 기상 주의사항을 제공하고 확인 이력 저장 |
| 🏷️ 보호구 NFC | 보호구 태그 토큰을 받아 종류·소유·사용중지 상태를 서버에서 검증하며 실제 착용을 증명한다고 표현하지 않음 |
| 📄 화물 문서 | DGD·C/I·MSDS를 등록하고 MSDS 8항의 보호구 요구사항을 정규화·검수 |
| ⚓ 작업 운영 | 컨테이너·복수 화물·작업·팀을 연결하고 혼재화물 보호구 조건을 병합 |

## 기술 스택

| 영역 | 스택 |
|---|---|
| 백엔드 | Rust · [vespera](https://github.com/dev-five-git/vespera) (axum 기반, 파일 라우팅 + OpenAPI 자동 생성) |
| DB | PostgreSQL (개발: SQLite) · SeaORM · [vespertide](https://github.com/dev-five-git/vespertide) (JSON 선언형 스키마·마이그레이션) |
| 프론트 | Next.js 16 · React 19 · devup-ui (작업자 Tauri 앱 `apps/front` / 관리자·게이트 웹 `apps/admin`) |
| 인증 | JWT + argon2 |
| 배포 | Docker Compose + nginx (온프레미스) |

## 프로젝트 구조

```
├── apis/api/            # Rust 백엔드 (포트 8000)
│   ├── models/          # DB 스키마 정의 (JSON — 진실의 원천, 25개 테이블)
│   ├── migrations/      # 자동 생성 마이그레이션 (앱 시작 시 자동 적용)
│   └── src/
│       ├── models/      # 자동 생성 SeaORM 엔티티 (직접 수정 금지)
│       ├── routes/      # API 핸들러 (파일 경로 = URL 경로)
│       └── utils/       # AppState, JWT, 인증 추출기
├── apps/front/          # 작업자용 Next.js + Tauri 모바일 앱 (포트 3000)
│   └── src-tauri/       # Android/iOS 앱 셸, 모바일 NFC 권한
├── apps/admin/          # 관리자용 웹 (포트 3001)
└── docker-compose.yml   # nginx + front/admin + api
```

## 실행

```bash
# 백엔드 (마이그레이션 자동 적용, 기본 SQLite)
cargo run                # http://localhost:8000 · Swagger UI: /docs

# 프론트 (front + admin 동시)
bun install
bun run dev

# 전체 (docker)
docker compose up

# 작업자 모바일 앱 개발
cd apps/front
bun run tauri:android:init   # 최초 1회: Android 프로젝트 생성
bun run tauri:android:dev    # Android 실기기/에뮬레이터 실행
```

### UI 화면 범위

현재 UI 브랜치는 API 연결 전 레이아웃과 상태 흐름을 검토할 수 있도록 대표 데이터를 포함한다.

- 작업자 `apps/front`: `/signin`, `/` — 오늘 작업 선택, 교육 적격성, 지침 확인, 휴대폰 보호구 NFC, 게이트 준비, 통과 기록, 내 정보. Next.js 정적 결과물을 Tauri 2 WebView로 패키징한다.
- 관리자 `apps/admin`: `/signin`, `/dashboard` — 운영 보드, 작업·팀 배정, 문서·화물, MSDS 보호구 검수, 교육, 지침·작업중지, 게이트 이력, 직원·조직
- 게이트 단말 `apps/admin`: `/gate-terminal` — 사원증 입력 대기, 판정 중, PASS/BLOCK 및 차단 사유

UI는 `@devup-ui/react`, `@devup-ui/next-plugin`, `ThemeScript`를 사용한다. 상태는 색상 배지보다 텍스트, 선 굵기, 간격과 정보 우선순위로 구분한다.

보호구 NFC는 모바일 앱에서 읽기 권한만 사용한다. 앱은 `assignmentId`, `equipmentCategory`, `tagToken`, `scannedAt`, 기기 채널을 API로 전달하고, 보호구 종류·등록 상태·사용중지 여부와 작업 적합성은 서버가 최종 판정한다. 일반 웹 프로덕션에서는 NFC 확인을 허용하지 않으며 개발 미리보기에서만 데모 태그가 생성된다.

상세 실행·인수인계 기준은 [`docs/frontend-handoff.md`](docs/frontend-handoff.md)를 참고한다.

**환경변수** (`apis/api/.env`): `DATABASE_URL`(기본 `sqlite:./test.db?mode=rwc`), `JWT_SECRET`, `PORT`(기본 8000)

### DB 스키마 변경 시

```bash
cargo install vespertide-cli   # 최초 1회
cd apis/api
# models/*.json 수정 후:
vespertide revision -m "변경 설명"    # 마이그레이션 생성
vespertide export --orm seaorm        # SeaORM 엔티티 재생성
```

⚠️ 규칙: FK 컬럼명은 반드시 `_id`로 끝낼 것 · `src/models/`는 직접 수정 금지 · 라우트 파일명의 언더스코어는 URL에서 하이픈으로 변환됨(`job_roles.rs` → `/job-roles`)

## API 현황

`cargo run` 후 **`http://localhost:8000/docs`** (Swagger UI)에서 전체 명세 확인.
프론트 연동은 자동 생성되는 `apps/*/openapi.json` → `schema.d.ts` 타입 기준.

| 도메인 | 엔드포인트 | 상태 |
|---|---|---|
| 인증 | `POST /auth/signin` (JWT 발급) | ✅ |
| 직원 | `GET/POST /employees`, `GET /employees/{id}` | ✅ |
| 부서/직무 | `GET/POST/PUT /departments`, `/job-roles` | ✅ |
| 출근 | `POST /attendances`, `GET /attendances/today`, `GET /attendances/today-instructions`, `GET /attendances/required-equipment`, `POST /attendances/ack`, `POST /attendances/equipment-complete` | ✅ |
| 장비 태깅 | `POST /equipment-checks` (돌려쓰기 차단) | ✅ |
| 게이트 | `POST /gate/verify` (사원증 NFC 검증) | ✅ |
| 승인 워크플로 · 화물 분석 · 법령 분석 · 배치 필터 · 리포트 | — | 🚧 [이슈 보드](../../issues) 참고 |

## 컨벤션

- **브랜치**: git flow — `develop` 기준 `feature/*`, `fix/*`
- **커밋**: `타입: 요약 (#이슈번호)` 한국어, 현재형 — `feat` `fix` `refactor` `chore` `docs` `test` `style` `ci` `perf`
- **이슈 → 브랜치 → 커밋(#이슈) → PR(Closes #이슈) → develop 머지**
- pre-commit: `oxlint` + `cargo clippy -D warnings` + `cargo fmt --check`

### 코드 주석

- 코드가 이미 보여주는 동작을 반복하지 않고 비즈니스 정책·보안 경계·기술적 제약의 이유를 설명한다.
- 장기 작업은 일반 주석으로 숨기지 않고 GitHub Issue로 등록한다.
- `TODO`가 필요한 경우 `TODO(#이슈번호): 이유` 형식을 사용하고 해결 즉시 제거한다.
