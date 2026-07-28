# PortDocAI

> 항만 안전 게이트키핑 플랫폼 — 입고 화물 정보와 실시간 개정 법령을 기반으로,
> 작업자가 **안전수칙을 인지하고 필수 장비를 착용했음이 검증되어야만** 작업 투입이
> 가능하도록 강제하는 시스템

**Team Portsync** — Port(항만) × Sync(동기화): 사람, 시스템, 안전을 하나로 연결합니다.

## 왜 필요한가

- 안전사고 대부분은 "몰라서" 또는 "깜빡해서" 발생하는 휴먼 에러 → 시스템이 **물리적으로 우회 불가능한 체크포인트**를 만들면 예방 가능
- 산안법·항만안전특별법·KOSHA Guide는 수시로 개정(연 300건+)되어 관리자가 수동 추적 불가능
- 기업의 안전교육·고지 의무 이행을 **위변조 없는 로그**로 증명 → 사고 시 면책 근거

## 핵심 기능

| 기능 | 설명 |
|---|---|
| 🚪 출근 게이트키핑 | 지침 확인 + 장비 착용 + 승인, 3조건 충족 시에만 게이트 통과 |
| 📋 안전수칙 인지 강제 | 팝업 최하단 스크롤 완료 후에만 확인 가능, 인지 로그 append-only 기록 |
| 🏷️ NFC 장비 검증 | 장비별 NFC 태깅, 장비 돌려쓰기 자동 차단 |
| 📄 법령·공문 분석 | HWP/PDF 공문에서 안전수칙·과태료 조항 자동 추출, 개정 diff |
| ⚓ 화물 문서 분석 | B/L·DGD에서 UN No./HS Code 추출 → IMDG Class별 지침·장비 자동 매핑 |
| 🌏 접근성 | 작업자 모국어 번역, TTS, 점자 병기 (산안법 37조) |

## 기술 스택

| 영역 | 스택 |
|---|---|
| 백엔드 | Rust · [vespera](https://github.com/dev-five-git/vespera) (axum 기반, 파일 라우팅 + OpenAPI 자동 생성) |
| DB | PostgreSQL (개발: SQLite) · SeaORM · [vespertide](https://github.com/dev-five-git/vespertide) (JSON 선언형 스키마·마이그레이션) |
| 프론트 | Next.js 16 · React 19 · devup-ui (사용자 `apps/front` / 관리자 `apps/admin`) |
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
├── apps/front/          # 작업자용 웹 (포트 3000)
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
```

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

주요 기술 선택과 정책의 근거는 [`ARCHITECTURE_DECISIONS.md`](ARCHITECTURE_DECISIONS.md)에 누적한다.
