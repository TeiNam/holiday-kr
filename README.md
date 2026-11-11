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

Temporary holidays are announced irregularly. New data ships as a patch release, so run `cargo update -p holiday-kr` to pick it up.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Holiday data: 한국천문연구원 특일 정보 (공공데이터포털, https://www.data.go.kr/data/15012690/openapi.do).

---

## 한국어

대한민국 공휴일(국경일·공휴일·대체공휴일·임시공휴일) 조회 Rust 라이브러리입니다. 한국천문연구원 특일 정보를 crate 안에 내장해 **API 키·네트워크·의존성 없이** 동작합니다.

- 지원 범위: `FIRST_YEAR`(2004) ~ `LAST_YEAR`(현재 2028). 범위 밖 연도는 `None` 을 돌려줍니다.
- 주말은 공휴일로 치지 않습니다.
- 명칭은 공공 데이터 그대로이되 `기독탄신일` → `성탄절`, `노동절` → `근로자의 날`, `신정` → `1월1일` 로 통일합니다.

### 설치

```sh
cargo add holiday-kr
```

### 사용

```rust
use holiday_kr::{holidays_between, holidays_in_month, holidays_on, is_holiday};

assert_eq!(is_holiday(2027, 2, 9), Some(true)); // 대체공휴일(설날)
assert_eq!(is_holiday(1999, 1, 1), None);       // 지원 범위 밖

// 한 날짜에 여러 항목이 있을 수 있어 슬라이스로 돌려준다
assert_eq!(holidays_on(2027, 5, 1)[0].name, "근로자의 날");

let february = holidays_in_month(2027, 2);
let chuseok = holidays_between(20270914, 20270916); // YYYYMMDD, 양 끝 포함
```

### 업데이트

임시공휴일이 지정되면 패치 버전으로 배포합니다. `cargo update -p holiday-kr` 로 받으세요.
