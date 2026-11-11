# holiday-kr

[![crates.io](https://img.shields.io/crates/v/holiday-kr.svg)](https://crates.io/crates/holiday-kr)
[![docs.rs](https://docs.rs/holiday-kr/badge.svg)](https://docs.rs/holiday-kr)

South Korean public holidays for Rust — national holidays, substitute holidays (대체공휴일) and temporary holidays (임시공휴일).

Holiday data from the Korea Astronomy and Space Science Institute (KASI, via data.go.kr) is embedded in the crate, so it works **without an API key, network access, or dependencies**.

[한국어](#한국어)

- Coverage: `FIRST_YEAR` (2004) to `LAST_YEAR` (currently 2028). The range follows whatever the official API provides.
- Weekends are not counted as holidays.
- Names are kept as published, except: `기독탄신일` → `성탄절`, `노동절` → `근로자의 날`, `신정` → `1월1일` (substitute-holiday names included).

## Install

```sh
cargo add holiday-kr
```

## Usage

```rust
use holiday_kr::{holidays_between, holidays_in_month, holidays_on, is_holiday};

// Out-of-range years return None instead of a silent `false`.
assert_eq!(is_holiday(2027, 2, 9), Some(true)); // 대체공휴일(설날)
assert_eq!(is_holiday(1999, 1, 1), None);

// A single date can have several entries, so a slice is returned.
assert_eq!(holidays_on(2027, 5, 1)[0].name, "근로자의 날");

// By month, or by an inclusive YYYYMMDD range.
let february = holidays_in_month(2027, 2);
let chuseok = holidays_between(20270914, 20270916);
```

## Updates

Temporary holidays are announced irregularly. A scheduled workflow regenerates the data weekly and opens a PR with a patch version bump; merging it publishes the new version. Run `cargo update -p holiday-kr` to pick it up.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Holiday data: 한국천문연구원 특일 정보 (공공데이터포털, https://www.data.go.kr/data/15012690/openapi.do).

---

## 한국어

대한민국 공휴일(국경일·공휴일·대체공휴일·임시공휴일) 조회 Rust 라이브러리입니다. 한국천문연구원 특일 정보를 crate 안에 내장해 **API 키·네트워크·의존성 없이** 동작합니다.

### 데이터 갱신 (관리자용)

임시공휴일 지정 등으로 데이터가 바뀌면 `src/data.rs` 를 다시 생성해 새 패치 버전으로 배포합니다.

**자동**: `update-data.yml` 이 매주 월요일 03:00(KST)에 데이터를 다시 받아, 바뀐 게 있으면 패치 버전과 CHANGELOG 를 올린 PR 을 엽니다. PR 을 머지하면 `release.yml` 이 crates.io 배포와 GitHub Release 생성을 합니다. 수동 실행은 Actions → *Update holiday data* → *Run workflow*.

**수동**:

```sh
DATA_GO_KR_SERVICE_KEY=<디코딩 키> cargo run -p gen
cargo test --workspace
```

생성기는 모든 연도를 받은 뒤에만 파일을 씁니다. 올해 이전 연도가 0건이거나 API 오류가 나면 실패하고 기존 데이터를 그대로 둡니다.

### 최초 설정

1. **GitHub Secret**: *Settings → Secrets and variables → Actions* 에 `DATA_GO_KR_SERVICE_KEY` (data.go.kr **디코딩 키**) 등록.
2. **PR 생성 권한**: *Settings → Actions → General → Workflow permissions* 에서 *Allow GitHub Actions to create and approve pull requests* 켜기.
3. **첫 배포는 수동**: crates.io Trusted Publishing 은 이미 존재하는 crate 에만 설정할 수 있습니다.
   ```sh
   cargo login            # crates.io API 토큰
   cargo publish -p holiday-kr
   ```
4. **Trusted Publishing 설정**: crates.io 의 crate 페이지 → *Settings → Trusted Publishing* 에서 GitHub 저장소 `TeiNam/holiday-kr`, 워크플로 `release.yml` 등록. 이후 배포에는 토큰이 필요 없습니다.

### 키 갱신 (1년마다)

data.go.kr 에서 활용 기간을 연장하거나 키를 재발급한 뒤 `DATA_GO_KR_SERVICE_KEY` 시크릿만 갱신하면 됩니다. 키는 생성기에서만 쓰이고 라이브러리 사용자에게는 필요 없습니다.

### 구조

| 경로 | 역할 |
|---|---|
| `src/lib.rs` | 공개 API (조회 함수) |
| `src/data.rs` | 생성된 공휴일 테이블 (직접 수정 금지) |
| `gen/` | data.go.kr 에서 받아 `src/data.rs` 를 만드는 생성기 (crate 에 포함되지 않음) |
| `.github/workflows/update-data.yml` | 주간 데이터 갱신 PR |
| `.github/workflows/release.yml` | crates.io 배포 + GitHub Release |
