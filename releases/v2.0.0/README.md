# VibeCoding Rust Scheduler v2.0.0

> 심리학 기반 하루 일정 관리 프로그램

## 📥 다운로드

### Scheduler.exe (7.2MB)
**GUI 버전** - 일반 사용자용

- Windows 10/11 지원
- WebView2 런타임 필요 (Windows 10/11에 기본 포함)
- 더블클릭으로 바로 실행

## 🚀 실행 방법

### 1. 다운로드
이 폴더의 `Scheduler.exe` 파일을 다운로드하세요.

### 2. 실행
다운로드한 파일을 더블클릭하면 바로 실행됩니다.

**Windows Defender 경고가 뜨면:**
1. "추가 정보" 클릭
2. "실행" 버튼 클릭

### 3. 첫 실행
- 앱이 실행되면 자동으로 오늘 날짜의 스케줄이 생성됩니다
- 왼쪽 사이드바에서 날짜를 선택할 수 있습니다

## ✨ 주요 기능

### 📅 스케줄 관리
- 작업 추가/수정/삭제
- 드래그앤드롭으로 시간 조정
- 작업 상태 관리 (대기/진행중/완료/일시정지)

### ⏱️ 타임라인 뷰
- 24시간 시각화
- 실시간 현재 시간 표시
- 작업별 색상 구분

### 📊 대시보드
- 오늘의 완료율
- 시간 효율성 분석
- 15가지 심리학 원리 적용 현황

### 🍅 포모도로 타이머
- 25분 작업 + 5분 휴식
- 집중도 평가 (1-5점)
- 세션 진행 상황 추적

### 💰 책임감 시스템
- 시간당 가치 설정
- 벌어들인 돈 / 낭비한 돈 계산
- 효율성 보너스/페널티

### 🤖 AI 연동
- Claude Code CLI 지원
- 일정 조언 및 분석
- 컨텍스트 기반 제안

## 📁 데이터 저장 위치

프로그램 데이터는 다음 위치에 저장됩니다:

```
%APPDATA%\com.scheduler.app\
└── schedules\
    ├── 2025-11-03.json
    ├── 2025-11-04.json
    └── ...
```

## 🔧 문제 해결

### WebView2 오류
Windows 10/11에는 WebView2가 기본으로 설치되어 있습니다.
만약 오류가 발생하면 [Microsoft WebView2 다운로드](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

### 실행이 안 되는 경우
1. Windows Defender가 차단하는지 확인
2. 파일을 다른 위치로 이동 후 실행
3. 관리자 권한으로 실행

### 데이터 백업
`%APPDATA%\com.scheduler.app\schedules\` 폴더를 복사해두세요.

## 📝 업데이트 내역

### v2.0.0 (2025-11-03)
- ✅ 타임라인 작업 표시 버그 수정
- ✅ 디버그 로그 추가
- ✅ 민감정보 제거 (환경 변수 기반으로 변경)
- ✅ 대시보드 UI 통일 (심리학 원리 테두리 제거)
- ✅ 15가지 심리학 원리 적용
- ✅ Multi-AI 지원 (Claude/Copilot CLI)
- ✅ 한국어 UI

## 🔗 링크

- **GitHub**: https://github.com/Taeksu-Kim/vibecoding-rust-scheduler
- **이슈 리포트**: https://github.com/Taeksu-Kim/vibecoding-rust-scheduler/issues

## 📄 라이선스

MIT License

---

**만든 사람**: Taeksu Kim
**제작 도구**: 🤖 Claude Code
