# holiday-kr

대한민국 공휴일(국경일·공휴일·대체공휴일·임시공휴일) 조회 Rust 라이브러리.

한국천문연구원 특일 정보(data.go.kr `getRestDeInfo`)를 crate 안에 내장하므로 **API 키·네트워크·DB 없이** 동작하고, 의존성도 없습니다.

- 데이터 범위: `FIRST_YEAR`(2004) ~ `LAST_YEAR`(현재 2028). 범위는 API 가 제공하는 만큼 자동으로 늘어납니다.
- 주말은 공휴일로 치지 않습니다.
- 명칭 정규화: `기독탄신일` → `성탄절`, `노동절` → `근로자의 날` (대체공휴일 명칭 포함). 나머지는 API 원문 그대로입니다.

## 설치

```toml
[dependencies]
holiday-kr = { git = "https://github.com/TeiNam/holiday-kr" }
```

## 사용

```rust
use holiday_kr::{holidays_between, holidays_in_month, holidays_on, is_holiday};

// 공휴일 여부. 데이터 범위 밖 연도는 None (조용히 false 를 주지 않음)
assert_eq!(is_holiday(2027, 2, 9), Some(true)); // 대체공휴일(설날)
assert_eq!(is_holiday(1999, 1, 1), None);

// 같은 날 여러 건이 있을 수 있어 슬라이스로 반환
let h = holidays_on(2027, 5, 1);
assert_eq!(h[0].name, "근로자의 날");

// 월별 / 기간별 (YYYYMMDD, 양끝 포함)
let feb = holidays_in_month(2027, 2);
let chuseok = holidays_between(20270914, 20270916);
```

## 데이터 갱신

임시공휴일 지정 등으로 데이터가 바뀌면 `src/data.rs` 를 다시 생성해 새 버전을 반영합니다.

**자동**: `.github/workflows/update-data.yml` 이 매주 월요일 03:00(KST)에 데이터를 다시 받아, 바뀐 게 있으면 PR 을 엽니다. 수동 실행은 Actions 탭의 *Update holiday data* → *Run workflow*.
사용하는 쪽은 PR 머지 후 `cargo update -p holiday-kr` 만 하면 됩니다.

**수동**:

```bash
DATA_GO_KR_SERVICE_KEY=<디코딩 키> cargo run -p gen
cargo test --workspace
```

생성기는 모든 연도를 받은 뒤에만 파일을 쓰고, 올해 이전 연도가 0건이거나 API 오류가 나면 실패합니다. 이때 기존 데이터는 그대로 남습니다.

### 최초 설정 / 키 갱신 (1년마다)

1. [data.go.kr 특일 정보](https://www.data.go.kr/data/15012690/openapi.do)에서 키를 발급하거나 연장합니다.
2. 레포의 *Settings → Secrets and variables → Actions* 에서 `DATA_GO_KR_SERVICE_KEY` 를 등록하거나 갱신합니다. **디코딩 키**를 넣으세요.
3. *Settings → Actions → General → Workflow permissions* 에서 **Allow GitHub Actions to create and approve pull requests** 를 켭니다 (최초 1회).

키는 생성기에서만 쓰입니다. 라이브러리 사용자에게는 키가 필요 없습니다.

## 구조

| 경로 | 역할 |
|---|---|
| `src/lib.rs` | 공개 API (조회 함수) |
| `src/data.rs` | 생성된 공휴일 테이블 (직접 수정 금지) |
| `gen/` | data.go.kr 에서 받아 `src/data.rs` 를 만드는 생성기 (배포되지 않음) |
