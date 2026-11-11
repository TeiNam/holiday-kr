//! 대한민국 공휴일 조회 라이브러리.
//!
//! 한국천문연구원 특일 정보(data.go.kr `getRestDeInfo`)를 crate 안에 내장하므로
//! API 키나 네트워크 없이 동작한다. 주말은 공휴일로 치지 않는다.
//!
//! ```
//! use holiday_kr::{holidays_on, is_holiday};
//!
//! assert_eq!(is_holiday(2027, 12, 25), Some(true));
//! assert_eq!(holidays_on(2027, 12, 25)[0].name, "성탄절");
//! assert_eq!(is_holiday(2027, 12, 24), Some(false));
//! // 내장 데이터 범위 밖의 연도는 알 수 없으므로 None
//! assert_eq!(is_holiday(1999, 1, 1), None);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod data;

/// 내장 데이터의 첫 연도.
pub const FIRST_YEAR: u16 = data::FIRST_YEAR;
/// 내장 데이터의 마지막 연도. 임시공휴일 등은 발표 후 데이터 갱신으로 추가된다.
pub const LAST_YEAR: u16 = data::LAST_YEAR;

/// 공휴일 한 건.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holiday {
    /// 날짜 (YYYYMMDD 정수, 예: 20270101)
    pub locdate: u32,
    /// 명칭 (예: "설날", "대체공휴일(성탄절)")
    pub name: &'static str,
    /// 공공기관 휴일 여부
    pub is_holiday: bool,
}

impl Holiday {
    /// 연도
    pub const fn year(&self) -> u16 {
        (self.locdate / 10000) as u16
    }

    /// 월 (1~12)
    pub const fn month(&self) -> u8 {
        (self.locdate / 100 % 100) as u8
    }

    /// 일 (1~31)
    pub const fn day(&self) -> u8 {
        (self.locdate % 100) as u8
    }
}

/// 연도가 내장 데이터 범위 안인지 여부.
pub const fn is_covered(year: u16) -> bool {
    FIRST_YEAR <= year && year <= LAST_YEAR
}

const fn ymd(year: u16, month: u8, day: u8) -> u32 {
    year as u32 * 10000 + month as u32 * 100 + day as u32
}

/// `start` ~ `end`(YYYYMMDD, 양끝 포함) 사이의 공휴일 목록 (날짜 오름차순).
pub fn holidays_between(start: u32, end: u32) -> &'static [Holiday] {
    let all = data::HOLIDAYS;
    let lo = all.partition_point(|h| h.locdate < start);
    let hi = all.partition_point(|h| h.locdate <= end);
    &all[lo..hi.max(lo)]
}

/// 특정 연도의 공휴일 목록.
pub fn holidays_in_year(year: u16) -> &'static [Holiday] {
    holidays_between(ymd(year, 1, 1), ymd(year, 12, 31))
}

/// 특정 연/월의 공휴일 목록.
pub fn holidays_in_month(year: u16, month: u8) -> &'static [Holiday] {
    holidays_between(ymd(year, month, 1), ymd(year, month, 31))
}

/// 특정 날짜의 공휴일 목록 (같은 날 여러 건일 수 있음).
pub fn holidays_on(year: u16, month: u8, day: u8) -> &'static [Holiday] {
    let d = ymd(year, month, day);
    holidays_between(d, d)
}

/// 특정 날짜가 공휴일인지 여부. 데이터 범위 밖의 연도면 `None`.
pub fn is_holiday(year: u16, month: u8, day: u8) -> Option<bool> {
    is_covered(year).then(|| holidays_on(year, month, day).iter().any(|h| h.is_holiday))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_is_sorted_and_every_year_present() {
        assert!(
            data::HOLIDAYS
                .windows(2)
                .all(|w| w[0].locdate <= w[1].locdate)
        );
        for year in FIRST_YEAR..=LAST_YEAR {
            assert!(!holidays_in_year(year).is_empty(), "{year}년 데이터 없음");
        }
    }

    #[test]
    fn renamed_holidays() {
        assert_eq!(holidays_on(2027, 5, 1)[0].name, "근로자의 날");
        assert_eq!(holidays_on(2027, 5, 3)[0].name, "대체공휴일(근로자의 날)");
        assert_eq!(holidays_on(2027, 12, 27)[0].name, "대체공휴일(성탄절)");
        assert_eq!(holidays_in_month(2004, 1)[0].name, "1월1일");
        let has_raw_name = |h: &Holiday| {
            ["기독탄신일", "노동절", "신정"]
                .iter()
                .any(|n| h.name.contains(n))
        };
        assert!(!data::HOLIDAYS.iter().any(has_raw_name));
    }

    #[test]
    fn lookups_2027() {
        assert_eq!(holidays_in_year(2027).len(), 24);
        let feb: Vec<_> = holidays_in_month(2027, 2).iter().map(|h| h.day()).collect();
        assert_eq!(feb, [6, 7, 8, 9]);
        assert_eq!(holidays_between(20270914, 20270916).len(), 3);
        assert_eq!(is_holiday(2027, 2, 9), Some(true));
        assert_eq!(is_holiday(2027, 2, 10), Some(false));
        assert_eq!(is_holiday(FIRST_YEAR - 1, 1, 1), None);
        assert_eq!(is_holiday(LAST_YEAR + 1, 1, 1), None);
    }

    #[test]
    fn empty_and_reversed_ranges() {
        assert!(holidays_between(20270102, 20270205).is_empty());
        assert!(holidays_between(20271231, 20270101).is_empty());
    }
}
