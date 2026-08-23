//! 통합 테스트 하네스스용 라이브러리 타깃 (FR-D5, #42)
//!
//! 바이너리 전용 크레이트였던 기존 구조를 유지하면서, 통합 테스트가
//! 실제 라우트 핸들러·extractor·AppState를 프로세스 안에서 조립해 쓸 수 있도록
//! 최소 공개 범위만 노출한다. 라우트 수집은 그대로 `vespera::vespera!` 매크로가 담당하며
//! (main.rs), 테스트에서는 동일한 `#[vespera::route]` 핸들러를 직접 Router에 연결한다.

pub mod config;
// 자동 생성 코드(vespertide export)라 미사용 항목 경고를 모듈 단위로 허용
#[allow(dead_code)]
pub mod models;
pub mod routes;
pub mod utils;
