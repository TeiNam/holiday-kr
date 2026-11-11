//! data.go.kr 특일 정보에서 공휴일을 받아 `src/data.rs` 를 다시 생성한다.
//!
//! 사용: `DATA_GO_KR_SERVICE_KEY=<디코딩 키> cargo run -p gen`
//! 모든 연도를 받은 뒤에만 파일을 쓰므로, 호출이 하나라도 실패하면 기존 데이터는 그대로 남는다.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;

const API_URL: &str =
    "https://apis.data.go.kr/B090041/openapi/service/SpcdeInfoService/getRestDeInfo";
/// API 가 데이터를 제공하는 첫 연도 (2003년 이전은 0건)
const START_YEAR: u16 = 2004;
/// 미래 연도는 이만큼까지만 탐색 (보통 1~2년 뒤까지만 데이터가 있음)
const MAX_YEARS_AHEAD: u16 = 5;
/// 연도 전체 공휴일은 수십 건이라 페이징 불필요
const NUM_OF_ROWS: &str = "100";
/// data.go.kr 가 가끔 응답 없이 연결을 붙잡고 있어서, 요청마다 상한을 둔다
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
/// 일시적 무응답·오류 대비 연도별 최대 시도 횟수
const MAX_ATTEMPTS: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);

/// 공공 API 명칭 → 저장용 명칭. 부분 치환이라 "대체공휴일(노동절)" 도 함께 바뀐다.
/// 1월 1일은 연도에 따라 "신정"/"1월1일" 로 섞여 와서 "1월1일" 로 통일한다.
const NAME_REPLACEMENTS: [(&str, &str); 3] = [
    ("기독탄신일", "성탄절"),
    ("노동절", "근로자의 날"),
    ("신정", "1월1일"),
];

#[derive(Debug, PartialEq)]
struct Row {
    locdate: u32,
    name: String,
    is_holiday: bool,
}

fn normalize_name(raw: &str) -> String {
    NAME_REPLACEMENTS
        .iter()
        .fold(raw.trim().to_string(), |name, (from, to)| {
            name.replace(from, to)
        })
}

/// items 는 0건이면 "", 1건이면 item 이 객체, 여러 건이면 배열로 온다.
fn item_list(items: &Value) -> Vec<&Value> {
    match items.get("item") {
        Some(Value::Array(list)) => list.iter().collect(),
        Some(obj @ Value::Object(_)) => vec![obj],
        _ => Vec::new(),
    }
}

fn parse_row(item: &Value) -> Result<Row, String> {
    let locdate = match &item["locdate"] {
        Value::Number(n) => n.as_u64().map(|v| v as u32),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
    .filter(|d| (10000101..=99991231).contains(d))
    .ok_or_else(|| format!("locdate 형식 오류: {item}"))?;
    let name = item["dateName"]
        .as_str()
        .ok_or_else(|| format!("dateName 없음: {item}"))?;
    Ok(Row {
        locdate,
        name: normalize_name(name),
        is_holiday: item["isHoliday"]
            .as_str()
            .is_some_and(|v| v.eq_ignore_ascii_case("Y")),
    })
}

/// 응답 본문을 검증하고 행 목록으로 변환.
fn parse_body(body: &str) -> Result<Vec<Row>, String> {
    // 키 오류 등은 XML 에러 응답으로 온다
    let json: Value = serde_json::from_str(body).map_err(|_| {
        let head: String = body.chars().take(200).collect();
        format!("JSON 파싱 실패 (서비스 키를 확인하세요): {head}")
    })?;
    let response = &json["response"];
    let code = response["header"]["resultCode"].as_str().unwrap_or("?");
    if code != "00" && code != "0000" {
        return Err(format!(
            "API 오류 [{code}] {}",
            response["header"]["resultMsg"]
        ));
    }
    let body = &response["body"];
    let rows = item_list(&body["items"])
        .into_iter()
        .map(parse_row)
        .collect::<Result<Vec<_>, _>>()?;
    let total = body["totalCount"].as_u64().unwrap_or(0) as usize;
    if rows.len() != total {
        return Err(format!(
            "totalCount({total}) 와 받은 건수({}) 불일치",
            rows.len()
        ));
    }
    Ok(rows)
}

fn fetch_year(agent: &ureq::Agent, key: &str, year: u16) -> Result<Vec<Row>, String> {
    let body = agent
        .get(API_URL)
        .query("serviceKey", key)
        .query("solYear", year.to_string())
        .query("numOfRows", NUM_OF_ROWS)
        .query("_type", "json")
        .call()
        // 에러 메시지에 키가 담긴 URL 이 섞이지 않도록 원인 종류만 남긴다
        .map_err(|e| format!("{year}년 API 호출 실패: {}", short_error(&e)))?
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("{year}년 응답 읽기 실패: {}", short_error(&e)))?;
    parse_body(&body).map_err(|e| format!("{year}년: {e}"))
}

/// 마지막 시도 전까지는 실패를 로그만 남기고 다시 시도한다.
fn with_retry<T>(mut f: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    for attempt in 1..MAX_ATTEMPTS {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => eprintln!("{e} — 재시도 ({attempt}/{})", MAX_ATTEMPTS - 1),
        }
        std::thread::sleep(RETRY_DELAY);
    }
    f()
}

fn short_error(e: &ureq::Error) -> String {
    match e {
        ureq::Error::StatusCode(code) => format!("HTTP {code}"),
        ureq::Error::Timeout(_) => "timeout".into(),
        ureq::Error::Io(io) => format!("io: {}", io.kind()),
        _ => "network error".into(),
    }
}

// shortcut: 평균 그레고리력 연 길이로 계산해 1월 1일 전후 하루 정도 오차가 있음. 검증 기준으로만 쓰므로 무해.
fn current_year() -> u16 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("시스템 시각 오류")
        .as_secs();
    (1970 + secs / 31_556_952) as u16
}

/// 올해까지는 빈 연도가 있으면 실패, 미래 연도는 첫 빈 연도에서 멈춘다.
fn fetch_all(key: &str) -> Result<Vec<Row>, String> {
    let this_year = current_year();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build()
        .into();
    let mut rows = Vec::new();
    for year in START_YEAR..=this_year + MAX_YEARS_AHEAD {
        let year_rows = with_retry(|| fetch_year(&agent, key, year))?;
        eprintln!("{year}: {}건", year_rows.len());
        if year_rows.is_empty() {
            if year <= this_year {
                return Err(format!(
                    "{year}년 데이터가 0건입니다 (API 이상 가능성, 기존 데이터 유지)"
                ));
            }
            break;
        }
        rows.extend(year_rows);
    }
    rows.sort_by(|a, b| (a.locdate, &a.name).cmp(&(b.locdate, &b.name)));
    rows.dedup();
    Ok(rows)
}

fn render(rows: &[Row]) -> String {
    let first = rows
        .first()
        .map_or(START_YEAR as u32, |r| r.locdate / 10000);
    let last = rows.last().map_or(START_YEAR as u32, |r| r.locdate / 10000);
    let mut out = format!(
        "// @generated by `cargo run -p gen` — 직접 수정하지 마세요.\n\
         use crate::Holiday;\n\n\
         pub(crate) const FIRST_YEAR: u16 = {first};\n\
         pub(crate) const LAST_YEAR: u16 = {last};\n\n\
         #[rustfmt::skip]\n\
         pub(crate) static HOLIDAYS: &[Holiday] = &[\n"
    );
    for r in rows {
        out += &format!(
            "    Holiday {{ locdate: {}, name: {:?}, is_holiday: {} }},\n",
            r.locdate, r.name, r.is_holiday
        );
    }
    out + "];\n"
}

fn main() -> Result<(), String> {
    let key = std::env::var("DATA_GO_KR_SERVICE_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or("DATA_GO_KR_SERVICE_KEY 환경 변수가 필요합니다 (data.go.kr 디코딩 키)")?;
    let rows = fetch_all(key.trim())?;
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../src/data.rs");
    std::fs::write(path, render(&rows)).map_err(|e| format!("{path} 쓰기 실패: {e}"))?;
    eprintln!("{}건 → src/data.rs", rows.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn retries_until_success_and_gives_up_after_max_attempts() {
        let mut calls = 0;
        let ok = with_retry(|| {
            calls += 1;
            if calls < 2 {
                Err("일시 오류".to_string())
            } else {
                Ok(calls)
            }
        });
        assert_eq!(ok, Ok(2));

        let mut fails = 0;
        let err: Result<(), String> = with_retry(|| {
            fails += 1;
            Err(format!("실패 {fails}"))
        });
        assert_eq!(err, Err(format!("실패 {MAX_ATTEMPTS}")));
    }

    #[test]
    fn normalizes_names() {
        assert_eq!(normalize_name(" 기독탄신일 "), "성탄절");
        assert_eq!(
            normalize_name("대체공휴일(기독탄신일)"),
            "대체공휴일(성탄절)"
        );
        assert_eq!(normalize_name("노동절"), "근로자의 날");
        assert_eq!(
            normalize_name("대체공휴일(노동절)"),
            "대체공휴일(근로자의 날)"
        );
        assert_eq!(normalize_name("신정"), "1월1일");
        assert_eq!(normalize_name("설날"), "설날");
    }

    #[test]
    fn item_list_handles_empty_single_and_many() {
        assert!(item_list(&json!("")).is_empty());
        assert_eq!(item_list(&json!({"item": {"a": 1}})).len(), 1);
        assert_eq!(item_list(&json!({"item": [{}, {}]})).len(), 2);
    }

    #[test]
    fn parses_body() {
        let body = json!({"response": {
            "header": {"resultCode": "00", "resultMsg": "OK"},
            "body": {"totalCount": 2, "items": {"item": [
                {"locdate": 20271225, "dateName": "기독탄신일", "isHoliday": "Y"},
                {"locdate": "20270501", "dateName": "노동절", "isHoliday": "N"}
            ]}}
        }});
        let rows = parse_body(&body.to_string()).unwrap();
        assert_eq!(
            rows[0],
            Row {
                locdate: 20271225,
                name: "성탄절".into(),
                is_holiday: true
            }
        );
        assert_eq!(
            rows[1],
            Row {
                locdate: 20270501,
                name: "근로자의 날".into(),
                is_holiday: false
            }
        );
    }

    #[test]
    fn rejects_error_and_mismatch() {
        let err = json!({"response": {"header": {"resultCode": "30", "resultMsg": "KEY ERROR"}}});
        assert!(parse_body(&err.to_string()).unwrap_err().contains("[30]"));
        let short = json!({"response": {"header": {"resultCode": "00"}, "body": {"totalCount": 3, "items": ""}}});
        assert!(parse_body(&short.to_string()).is_err());
        assert!(parse_body("<OpenAPI_ServiceResponse>").is_err());
    }
}
