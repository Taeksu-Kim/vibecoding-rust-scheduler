# Development Roadmap
# Daily Scheduler - 개발 계획서

## Overview

이 문서는 PRD를 기반으로 한 실제 개발 계획입니다.
단계별로 진행하며, 각 단계는 독립적으로 테스트 가능한 기능을 포함합니다.

---

## Phase 0: Project Setup & Foundation

### 0.1 프로젝트 초기화
- [ ] Cargo 프로젝트 생성 (`cargo new scheduler`)
- [ ] Git 저장소 초기화 및 .gitignore 설정
- [ ] 기본 프로젝트 구조 설계
  ```
  scheduler/
  ├── src/
  │   ├── main.rs
  │   ├── lib.rs
  │   ├── models/      # 데이터 모델
  │   ├── storage/     # 파일 저장
  │   ├── daemon/      # 백그라운드 데몬
  │   ├── cli/         # CLI 인터페이스
  │   ├── tui/         # TUI 위젯
  │   └── claude/      # Claude 연동
  ├── tests/
  ├── Cargo.toml
  └── README.md
  ```

### 0.2 Dependencies 추가
- [ ] 기본 라이브러리 추가 (Cargo.toml)
  ```toml
  [dependencies]
  # CLI
  clap = { version = "4.5", features = ["derive"] }

  # Serialization
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"

  # Time handling
  chrono = "0.4"

  # TUI (나중에 추가)
  # ratatui = "0.26"
  # crossterm = "0.27"

  # Error handling
  anyhow = "1.0"
  thiserror = "1.0"
  ```

### 0.3 기본 타입 정의
- [ ] `models/task.rs`: Task 구조체 정의
- [ ] `models/schedule.rs`: Schedule 구조체 정의
- [ ] `models/time_block.rs`: TimeBlock 구조체 정의
- [ ] 기본 enum 정의 (TaskStatus, TimeBlockType 등)

**산출물**: 빌드 가능한 기본 프로젝트 구조

---

## Phase 1: Core Data Layer

### 1.1 데이터 모델 구현

#### Task Model
- [ ] Task 구조체 완성
  ```rust
  pub struct Task {
      id: String,
      title: String,
      start_time: DateTime<Local>,
      end_time: DateTime<Local>,
      estimated_duration: Duration,
      actual_duration: Option<Duration>,
      status: TaskStatus,
      tags: Vec<String>,
      notes: Option<String>,
  }
  ```
- [ ] TaskStatus enum (Pending, InProgress, Completed, Paused, Skipped)
- [ ] Task 메서드 구현 (start, pause, complete, elapsed_time 등)

#### Schedule Model
- [ ] Schedule 구조체 (하루치 작업 모음)
- [ ] Schedule 메서드 (add_task, remove_task, get_current_task 등)
- [ ] 시간 충돌 검증 로직

#### Statistics Model
- [ ] DailyStats 구조체
- [ ] 완료율, 시간 정확도 계산 로직
- [ ] Streak 계산 로직

### 1.2 Storage Layer 구현

- [ ] `storage/mod.rs`: Storage trait 정의
- [ ] `storage/json_storage.rs`: JSON 파일 기반 저장소
  - [ ] 스케줄 저장 (save_schedule)
  - [ ] 스케줄 불러오기 (load_schedule)
  - [ ] 히스토리 저장 (날짜별)
- [ ] 데이터 디렉토리 구조
  ```
  ~/.config/scheduler/
  ├── current.json        # 오늘 스케줄
  ├── history/
  │   ├── 2025-10-30.json
  │   └── 2025-10-29.json
  └── stats.json          # 통계 데이터
  ```
- [ ] 파일 읽기/쓰기 에러 핸들링

### 1.3 Unit Tests
- [ ] Task 모델 테스트
- [ ] Schedule 로직 테스트
- [ ] Storage 테스트 (임시 디렉토리 사용)

**산출물**: 데이터 저장/불러오기가 가능한 코어 라이브러리

---

## Phase 2: Basic CLI

### 2.1 CLI 프레임워크 구축

- [ ] `cli/mod.rs`: CLI 구조 정의
- [ ] Clap을 이용한 명령어 파싱
  ```rust
  enum Commands {
      Add { title: String, start: String, end: String },
      List,
      Start { id: Option<String> },
      Pause,
      Complete,
      Status,
  }
  ```

### 2.2 기본 명령어 구현

- [ ] `sched add`: 작업 추가
  - [ ] 시간 파싱 (HH:MM 형식)
  - [ ] Schedule에 추가
  - [ ] 저장
- [ ] `sched list`: 오늘 스케줄 출력
  - [ ] 시간순 정렬
  - [ ] 상태별 색상 (간단한 ANSI 코드)
- [ ] `sched start [id]`: 작업 시작
  - [ ] ID 없으면 다음 작업 자동 시작
  - [ ] 상태 업데이트
- [ ] `sched pause`: 일시정지
- [ ] `sched complete`: 완료
- [ ] `sched status`: 현재 상태 출력

### 2.3 출력 포맷팅

- [ ] 테이블 형식 출력 (간단한 ASCII 테이블)
- [ ] 색상 코드 적용 (완료=녹색, 진행중=노란색, 대기=회색)
- [ ] 시간 포맷팅 (상대 시간: "2h 30m ago", "in 1h")

### 2.4 통합 테스트

- [ ] CLI 명령어 end-to-end 테스트
- [ ] 시나리오 테스트 (add → start → complete 플로우)

**산출물**: 기본적인 스케줄 관리가 가능한 CLI

---

## Phase 3: Background Daemon

### 3.1 Daemon 아키텍처 설계

- [ ] `daemon/mod.rs`: Daemon 구조 설계
- [ ] IPC 방식 결정 (Unix Socket / Named Pipe)
- [ ] 프로세스 관리 전략

### 3.2 Daemon 기본 기능

- [ ] Daemon 시작/중지/재시작
- [ ] PID 파일 관리 (`~/.config/scheduler/daemon.pid`)
- [ ] 시작 시 기존 daemon 체크
- [ ] 백그라운드 루프 구현

### 3.3 시간 추적

- [ ] 1분마다 현재 작업 체크
- [ ] 경과 시간 자동 업데이트
- [ ] 작업 시작/종료 시간 자동 기록
- [ ] 시간 초과 감지

### 3.4 알림 시스템

- [ ] 작업 시작 알림 (5분 전)
- [ ] 작업 종료 알림
- [ ] 시간 초과 경고
- [ ] OS 알림 연동 검토 (notify-rust)

### 3.5 IPC 통신

- [ ] CLI ↔ Daemon 통신 프로토콜
- [ ] 명령어 전달 (start, pause, complete 등)
- [ ] 상태 조회
- [ ] 에러 핸들링

### 3.6 Daemon 명령어

- [ ] `sched daemon start`
- [ ] `sched daemon stop`
- [ ] `sched daemon status`
- [ ] `sched daemon restart`

**산출물**: 백그라운드에서 자동으로 시간을 추적하는 Daemon

---

## Phase 4: Terminal Widget

### 4.1 Ratatui 기초

- [ ] Ratatui + Crossterm 의존성 추가
- [ ] 기본 TUI 앱 구조 생성
- [ ] Event loop 구현
- [ ] 터미널 설정 (raw mode, alternate screen)

### 4.2 Minimal Widget 구현

- [ ] `tui/widget.rs`: 작은 코너 위젯
- [ ] 레이아웃 (우측 상단 고정)
- [ ] 기본 정보 표시
  - 현재 시간
  - 진행률
  - 현재 작업
- [ ] 녹색 테마 적용

### 4.3 Widget 모드

- [ ] Minimal mode (3줄 정도)
- [ ] Expanded mode (Ctrl+Shift+S로 토글)
- [ ] 상태 전환 애니메이션 (부드럽게)

### 4.4 터미널 통합

- [ ] 기존 터미널 내용 유지
- [ ] 터미널 크기 변경 대응
- [ ] 다른 터미널 앱과 충돌 방지
- [ ] 백그라운드에서 위젯 업데이트

### 4.5 Widget 명령어

- [ ] `sched widget` (기본: 토글)
- [ ] `sched widget show`
- [ ] `sched widget hide`
- [ ] 위젯 실행 시 daemon 자동 시작

**산출물**: 터미널 우측 상단에 항상 표시되는 위젯

---

## Phase 5: Statistics & Progress Tracking

### 5.1 통계 수집

- [ ] DailyStats 저장/불러오기
- [ ] 완료율 계산
- [ ] 시간 정확도 계산 (예상 vs 실제)
- [ ] 집중 시간 (Deep Work) 추적
- [ ] 휴식 시간 추적

### 5.2 Streak 시스템

- [ ] 연속 달성 일수 계산
- [ ] 최고 기록 저장
- [ ] Streak 깨짐 조건 정의 (70% 미만 완료?)

### 5.3 통계 명령어

- [ ] `sched stats`: 오늘 통계
- [ ] `sched stats week`: 주간 통계
- [ ] `sched streak`: Streak 정보

### 5.4 통계 시각화 (CLI)

- [ ] ASCII 프로그레스 바
- [ ] 주간 차트 (간단한 막대 그래프)
- [ ] 요약 정보

**산출물**: Progress Monitoring 원칙 구현

---

## Phase 6: Time Management Features

### 6.1 Implementation Intentions

- [ ] If-Then 규칙 정의
- [ ] 시간 기반 자동 트리거
- [ ] 알림 메시지 커스터마이징

### 6.2 Time Awareness

- [ ] 예상 시간 vs 실제 시간 비교
- [ ] 정확도 피드백
- [ ] Planning Fallacy 경고

### 6.3 Fresh Start Effect

- [ ] 하루 구간 나누기 (오전/오후/저녁)
- [ ] 구간별 "새 시작" 메시지
- [ ] 실패한 작업 재시작 기능

### 6.4 Pomodoro Integration

- [ ] Pomodoro 타이머 옵션
- [ ] 25분 작업 + 5분 휴식
- [ ] 큰 시간 블록 내에서 Pomodoro 카운트
- [ ] `sched pomodoro start`

**산출물**: 심리학 원칙 3-4개 구현

---

## Phase 7: Claude Code Integration

### 7.1 Claude 연동 설계

- [ ] Claude API/CLI 호출 방식 결정
- [ ] 프롬프트 템플릿 시스템
- [ ] Context 수집 로직

### 7.2 Context Collection

- [ ] 현재 작업 정보
- [ ] Git 상태 (branch, recent commits)
- [ ] 현재 디렉토리
- [ ] 오늘 스케줄 요약

### 7.3 Claude 명령어 구현

#### `sched claude <question>`
- [ ] 질문 + 컨텍스트를 Claude에게 전달
- [ ] 응답 받아서 표시
- [ ] 대화 히스토리 저장 (optional)

#### `sched validate`
- [ ] 오늘 스케줄을 Claude에게 전송
- [ ] 스케줄 타당성 분석 요청
- [ ] 피드백 표시

#### `sched optimize`
- [ ] 현재 상황 (지연 등) 전달
- [ ] 남은 일정 재조정 제안 요청
- [ ] 제안 수용/거부 옵션

### 7.4 프롬프트 템플릿

- [ ] `templates/schedule_validation.txt`
- [ ] `templates/task_assistant.txt`
- [ ] `templates/optimization.txt`
- [ ] 변수 치환 시스템

### 7.5 응답 처리

- [ ] JSON 형식 응답 파싱
- [ ] 제안사항 자동 적용 옵션
- [ ] 에러 핸들링

**산출물**: Claude Code와 완전히 통합된 스케줄러

---

## Phase 8: Full TUI

### 8.1 Full Screen TUI

- [ ] `sched ui` 명령어
- [ ] 전체 화면 레이아웃 (PRD 4.3 참조)
- [ ] 3-column layout: Timeline | Details | Stats

### 8.2 Timeline View

- [ ] 시간축 그리기
- [ ] 시간 블록 시각화 (박스 그리기)
- [ ] 현재 시간 인디케이터
- [ ] 스크롤 가능

### 8.3 Interactive Features

- [ ] 키보드 내비게이션 (↑/↓)
- [ ] 작업 선택
- [ ] 상세 정보 표시
- [ ] 편집 모드 (E키)

### 8.4 Additional Views

- [ ] Stats View (S키)
- [ ] Claude Integration View (C키)
- [ ] Help View (?키)

### 8.5 애니메이션

- [ ] Progress bar 애니메이션
- [ ] Current time indicator 깜빡임
- [ ] Task completion 효과

**산출물**: 풍부한 기능을 가진 Full TUI

---

## Phase 9: Time Accountability System

### 9.1 시간 성과 추적 모델

- [ ] `TimeAccountability` 모델 생성
  - [ ] `earned_time`: 지켜진 시간 (분)
  - [ ] `wasted_time`: 낭비한 시간 (분)
  - [ ] `bonus_time`: 보너스 시간 (예상보다 빨리 완료)
  - [ ] `penalty_time`: 페널티 시간 (예상보다 늦게 완료)
  - [ ] `efficiency_score`: 시간 효율 점수 (0-100%)
- [ ] Task 완료 시 시간 성과 계산 로직
  - [ ] 시간 내 완료: `earned_time = estimated_duration`
  - [ ] 빨리 완료: `bonus_time = estimated - actual`
  - [ ] 늦게 완료: `penalty_time = actual - estimated`
  - [ ] 건너뜀/미완료: `wasted_time = estimated_duration`

### 9.2 일일/주간/월간 집계

- [ ] `DailyAccountability` 모델
  - [ ] 총 계획 시간
  - [ ] 총 지켜진 시간
  - [ ] 총 낭비한 시간
  - [ ] 효율 점수 계산
- [ ] 주간/월간 통계 집계
- [ ] 저장소에 히스토리 저장

### 9.3 CLI 명령어

- [ ] `sched report` - 시간 성과 리포트
  - [ ] 오늘의 성과 (earned/wasted/bonus/penalty)
  - [ ] 주간 성과 (`--week` 플래그)
  - [ ] 월간 성과 (`--month` 플래그)
- [ ] `sched efficiency` - 효율 점수 트렌드
  - [ ] 일일 효율 점수 그래프 (ASCII)
  - [ ] 주간 평균
  - [ ] 개선/하락 추세 표시

### 9.4 즉각적인 피드백

- [ ] Task 완료 시 즉시 성과 표시
  - [ ] "+45m earned ✓" (시간 내 완료)
  - [ ] "+15m bonus! 🎉" (예상보다 빨리)
  - [ ] "-10m penalty ⚠" (예상보다 늦게)
- [ ] Task 건너뜀 시 경고
  - [ ] "-60m wasted ✗" (명확한 손실 표시)
- [ ] 색상 코드: Green (earned), Yellow (penalty), Red (wasted)

### 9.5 TUI 통합

- [ ] Stats 패널에 시간 성과 추가
  - [ ] 오늘의 earned/wasted 시간
  - [ ] 효율 점수 프로그레스 바
- [ ] Details 패널에 개별 작업 성과 표시

**산출물**: 명확한 시간 책임성 시스템으로 행동 강화

---

## Phase 10: Polish & Production Ready

### 10.1 에러 처리

- [ ] 모든 에러 케이스 검토
- [ ] 친절한 에러 메시지
- [ ] 복구 전략 (corrupted data 등)

### 10.2 설정 시스템

- [ ] `~/.config/scheduler/config.toml`
- [ ] 기본 시간 블록 크기
- [ ] 색상 테마
- [ ] 알림 설정
- [ ] Claude API 키

### 10.3 문서화

- [ ] README.md 완성
- [ ] 설치 가이드
- [ ] 사용법 (--help 메시지)
- [ ] 예제 시나리오

### 10.4 성능 최적화

- [ ] Daemon 메모리 사용량 체크
- [ ] Widget 렌더링 성능
- [ ] 대용량 히스토리 처리

### 10.5 크로스 플랫폼 테스트

- [ ] macOS 테스트
- [ ] Linux 테스트
- [ ] Windows 테스트 (WSL 포함)

### 10.6 패키징

- [ ] Cargo 릴리즈 빌드 설정
- [ ] Binary 크기 최적화
- [ ] Install script 작성

**산출물**: 프로덕션 배포 가능한 v1.0

---

## Phase 11: Tauri GUI Client (Windows Desktop)

### 11.1 Tauri Setup

- [ ] Tauri 1.5.4 (stable) 프로젝트 초기화
- [ ] Rust backend 연동 (기존 lib 재사용)
- [ ] Frontend 프레임워크 선택 및 설정
  - [ ] React/Vue/Svelte 중 선택
  - [ ] TypeScript 설정
  - [ ] Tailwind CSS (녹색 테마)

### 11.2 Core UI Components

- [ ] Timeline View (시간축 시각화)
  - [ ] Drag & Drop으로 작업 시간 조정
  - [ ] 시간 블록 색상 코드 (상태별)
  - [ ] 현재 시간 인디케이터 (실시간)
- [ ] Task Editor
  - [ ] 작업 추가/수정 폼
  - [ ] 태그 자동완성
  - [ ] 시간 picker
- [ ] Dashboard
  - [ ] 오늘의 통계 카드
  - [ ] Time Accountability 시각화
  - [ ] Efficiency 그래프 (주간/월간)
  - [ ] Streak 표시

### 11.3 Advanced Features

- [ ] System Tray Integration
  - [ ] 우클릭 메뉴 (Quick Add, Status)
  - [ ] 알림 표시
  - [ ] 현재 작업 표시
- [ ] Notifications
  - [ ] Windows 네이티브 알림
  - [ ] 작업 시작 5분 전 알림
  - [ ] 작업 종료 알림
  - [ ] Pomodoro 타이머 알림
- [ ] Settings Panel
  - [ ] 테마 변경 (Green/Blue/Purple/Cyan)
  - [ ] 알림 설정
  - [ ] Daemon 설정
  - [ ] 기본 시간 블록 크기

### 11.4 Backend Integration

- [ ] IPC 통신 (Tauri Commands)
  - [ ] Schedule CRUD operations
  - [ ] Statistics queries
  - [ ] Config management
- [ ] Daemon 통합
  - [ ] Daemon 상태 모니터링
  - [ ] 자동 시작 옵션
- [ ] Real-time Updates
  - [ ] WebSocket or Polling
  - [ ] 실시간 시간 업데이트
  - [ ] 작업 상태 변경 감지

### 11.5 Visual Polish

- [ ] 애니메이션
  - [ ] 작업 완료 시 축하 애니메이션
  - [ ] Smooth transitions
  - [ ] Progress bar 애니메이션
- [ ] 차트 & 그래프
  - [ ] Efficiency 트렌드 라인 차트
  - [ ] Time Accountability 파이 차트
  - [ ] 주간 히트맵
- [ ] 반응형 레이아웃
  - [ ] 창 크기 조정 대응
  - [ ] 최소 창 크기 설정

### 11.6 Windows-Specific Features

- [ ] Start with Windows
  - [ ] 레지스트리 등록
  - [ ] 최소화된 상태로 시작
- [ ] Keyboard Shortcuts
  - [ ] Global hotkey (Ctrl+Shift+S)
  - [ ] 빠른 작업 추가 (Ctrl+N)
  - [ ] 현재 작업 시작/완료 (Ctrl+Space)
- [ ] File Association
  - [ ] .sched 파일 형식
  - [ ] 스케줄 파일 더블클릭으로 열기

### 11.7 Packaging & Distribution

- [ ] MSI Installer 생성
  - [ ] WiX Toolset 사용
  - [ ] 시작 메뉴 바로가기
  - [ ] 자동 업데이트 체크
- [ ] Code Signing
  - [ ] Windows SmartScreen 우회
  - [ ] 신뢰할 수 있는 게시자
- [ ] Auto-updater
  - [ ] Tauri updater 플러그인
  - [ ] GitHub Releases 연동

### 11.8 Testing

- [ ] Unit tests (Rust backend)
- [ ] Integration tests (IPC)
- [ ] E2E tests (Frontend)
- [ ] Windows 10/11 호환성 테스트

**산출물**: Windows용 네이티브 데스크톱 애플리케이션

---

## Phase 12: Future Advanced Features

### 12.1 Template System
- [ ] 스케줄 템플릿 저장/불러오기
- [ ] 반복 패턴 (매주 월요일 등)

### 12.2 External Integrations
- [ ] Google Calendar 동기화
- [ ] Slack/Discord 알림
- [ ] GitHub Issues 연동

### 12.3 더 많은 심리학 원칙
- [ ] Temptation Bundling 구현
- [ ] Commitment Devices (공개 선언)
- [ ] Accountability System (공유)

**산출물**: 확장 가능한 통합 시스템

---

## 개발 우선순위

### ✅ Completed (v1.0) - CLI/TUI Application
1. Phase 0-2: 기본 CLI
2. Phase 3: Daemon
3. Phase 4: Widget
4. Phase 5-6: 통계 & 시간 관리
5. Phase 7: Claude 연동
6. Phase 8: Full TUI
7. Phase 9: Time Accountability System
8. Phase 10: Polish & Production Ready

### Next (v2.0) - GUI Desktop Application
- Phase 11: Tauri GUI Client (Windows)
  - System tray integration
  - Visual timeline with drag & drop
  - Native notifications
  - MSI installer

### Future (v3.0) - Advanced Features
- Phase 12: External Integrations & Templates

---

## 개발 원칙

### 1. Incremental Development
- 각 Phase가 독립적으로 동작
- 이전 Phase가 완료되어야 다음 진행
- 매 Phase 후 테스트

### 2. Test-Driven
- Unit test 작성
- Integration test
- 실제 사용 시나리오 테스트

### 3. Documentation
- 코드 주석
- API 문서
- 사용자 문서

### 4. Git Workflow
- Feature branch 사용
- Phase별 merge
- 의미 있는 커밋 메시지

---

## 체크리스트 사용법

이 로드맵의 체크박스는 실제 개발 진행 상황을 추적하는 데 사용됩니다.

**완료 시**:
- `- [ ]` → `- [x]`로 변경
- Git commit 시 참조

**진행 중**:
- `- [ ]` → `- [🔄]` (optional)

**블로킹됨**:
- `- [ ]` → `- [⚠️]` (optional)

---

**Last Updated**: 2025-10-30
**Status**: Ready for Development
