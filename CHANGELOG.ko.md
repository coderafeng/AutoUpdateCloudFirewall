# 변경 이력

이 파일은 프로젝트의 중요한 변경 사항을 모두 기록합니다.

형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를 따르고, 버전 번호는
[유의적 버전](https://semver.org/lang/ko/)을 따릅니다.

[English](CHANGELOG.md) · [简体中文](CHANGELOG.zh-CN.md) · [日本語](CHANGELOG.ja.md) · 한국어

## [미배포]

## [1.0.0] - 2026-11-16

첫 안정 버전.

### 추가

- Aliyun ECS와 Tencent Cloud CVM 클라우드 계정. 계정마다 AccessKey, 리전, 사용 여부를
  따로 가지며, 한 계정으로 여러 보안 그룹 대상을 관리할 수 있습니다.
- 대상별 인바운드 규칙 줄: 프로토콜, 단일 포트 또는 범위
  (`5432`, `8000-8010`, `8000/8010`), 선택적 설명 키워드.
- 설명 키워드로 기존 규칙 인수. 키워드를 입력한 줄은 프로토콜과 포트가 같고 설명에 그
  키워드가 포함된 규칙을 인수하여 원본 주소만 다시 씁니다
  (Aliyun은 `ModifySecurityGroupRule`). 설명과 우선순위는 유지되고 중복 규칙은 생기지
  않습니다.
- 모호성 방지: 키워드가 여러 규칙과 일치하면 추측하지 않고 대상 전체를 오류로
  건너뜁니다.
- 키워드가 없는 줄은 설명 접두사(기본값 `AutoFirewall`)로 관리하며, 직접 만든 규칙은 절대
  수정하거나 삭제하지 않습니다.
- 드라이런: 계획만 로그에 기록하고 쓰기 API는 호출하지 않습니다.
- 트레이 연동: 창을 닫아도 작업자는 계속 실행되며, OS 시작 시 자동 실행과 최소화 시작을
  지원합니다(Windows).
- 활동 로그: UI에는 최근 500건, 전체 기록은
  `%LOCALAPPDATA%\AutoUpdateCloudFirewall\logs\app.log`에 추가됩니다.
- 4개 언어 UI(영어, 중국어 간체, 일본어, 한국어). 전환하면 창, 대화 상자, 트레이 메뉴,
  새 로그 줄에 즉시 반영됩니다.
- 비밀 정보는 OS 자격 증명 저장소에 보관하고, `config.json`에는 AccessKey ID만
  기록합니다.
- 릴리스 산출물은 플랫폼별로
  `auto-update-cloud-firewall-x86_64-{windows,darwin,linux}.exe` 이름으로 공개합니다.

### 변경

- 동기화는 **삭제 → 갱신 → 생성** 순서로 적용합니다. Aliyun은 다른 규칙과 완전히
  중복되는 상태로의 변경을 거부하므로(`InvalidOperation.RuleDuplicate`), 오래된 규칙을
  먼저 삭제한 뒤 인수한 규칙에 주소를 씁니다.
- 클라우드 오류는 잘린 JSON 대신 거부된 RAM 작업과 RequestId를 보고합니다.
- 동기화 간격은 최소 60초, 기본 300초로 검증합니다.
- 사이드바 하단과 설정 페이지에 저장소 링크와 MIT 라이선스를 표시합니다.

### 수정

- 긴 로그 줄이 카드 테두리를 넘지 않고 말줄임표로 잘립니다.
- 창 왼쪽 아래와 실행 파일 버전 리소스를 1.0.0으로 통일했습니다.

### 알려진 제한

- 1.0.0의 지원 플랫폼은 Windows 10/11입니다. macOS와 Linux는 같은 소스로 빌드했지만
  동일한 수준으로 검증하지 않았습니다.
- IPv4와 인바운드 규칙만 처리합니다.
- Tencent Cloud에는 규칙 단위 수정 API가 없어 원본 IP 변경은 "원래 설명으로 삭제 후
  재생성"이 됩니다. 규칙 ID는 바뀌지만 설명과 우선순위는 유지됩니다.
- 기록할 수 있는 주소는 현재 공인 주소뿐입니다. `0.0.0.0/0`으로 되돌리려면 콘솔에서
  직접 수정해야 합니다.

[미배포]: https://github.com/coderafeng/AutoUpdateCloudFirewall/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/coderafeng/AutoUpdateCloudFirewall/releases/tag/v1.0.0
