use crate::time::constants::*;
use crate::time::Instant;
use sbor::rust::fmt;
use sbor::rust::fmt::Display;
use sbor::rust::num::ParseIntError;
use sbor::rust::str::FromStr;
use sbor::rust::vec::Vec;
use sbor::*;

const UNIX_EPOCH_YEAR: u32 = 1970;

const SECONDS_IN_A_NON_LEAP_YEAR: i64 = 365 * 24 * 60 * 60;
const SECONDS_IN_A_LEAP_YEAR: i64 = 366 * 24 * 60 * 60;

const DAYS_PER_4Y: i64 = 365 * 4 + 1;
const DAYS_PER_100Y: i64 = 365 * 100 + 24;
const DAYS_PER_400Y: i64 = 365 * 400 + 97;

const LEAP_YEAR_DAYS_IN_MONTHS: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// A shift (in seconds) from the Unix epoch (1970-01-01 00:00:00)
/// to a base date that is a multiple of a 400-year leap cycle.
///
/// Used in `Instant` -> `UtcDateTime` conversion.
///
/// The date we're using is `2000-03-01 00:00:00`, for two reasons:
/// 1. It's a multiple of 400, to make it easier to work with leap years
/// 2. We're also shifting the month to 1st March, so that
///    the extra day on leap years is added to the last month (Feb),
///    not in the middle of a year (makes some calculations easier)
const SHIFT_FROM_UNIX_TIME_TO_MARCH_Y2K: i64 = 946684800 + 86400 * (31 + 29);

/// A minimum Unix timestamp value that is supported by `UtcDateTime`.
///
/// This value corresponds to a date of `1-1-1 00:00:00`. Year `0` isn't allowed.
const MIN_SUPPORTED_TIMESTAMP: i64 = -62135596800;

/// A maximum Unix timestamp value that is supported by `UtcDateTime`.
///
/// This value corresponds to a date of `4294967295-12-31 23:59:59`,
/// where year `4294967295` equals `u32::MAX`.
const MAX_SUPPORTED_TIMESTAMP: i64 = 135536014634284799;

#[derive(Sbor, PartialEq, Eq, Copy, Clone, Debug)]
pub enum DateTimeError {
    InvalidYear,
    InvalidMonth,
    InvalidDayOfMonth,
    InvalidHour,
    InvalidMinute,
    InvalidSecond,
    InstantIsOutOfRange,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for DateTimeError {}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateTimeError::InvalidYear =>
                write!(f, "Invalid year. Expected a value strictly greater than 0"),
            DateTimeError::InvalidMonth =>
                write!(f, "Invalid month. Expected a value between 1 (inclusive) and 12 (inclusive)"),
            DateTimeError::InvalidDayOfMonth =>
                write!(f, "Invalid day of month. Expected a value between 1 (inclusive) and, depending on a month, 28, 29 (Feb on a leap year), 30 or 31 (inclusive)"),
            DateTimeError::InvalidHour =>
                write!(f, "Invalid hour. Expected a value between 0 (inclusive) and 23 (inclusive)"),
            DateTimeError::InvalidMinute =>
                write!(f, "Invalid minute. Expected a value between 0 (inclusive) and 59 (inclusive)"),
            DateTimeError::InvalidSecond =>
                write!(f, "Invalid second. Expected a value between 0 (inclusive) and 59 (inclusive)"),
            DateTimeError::InstantIsOutOfRange =>
                write!(f, "Instant out of supported range [{}, {}]", MIN_SUPPORTED_TIMESTAMP, MAX_SUPPORTED_TIMESTAMP),
        }
    }
}

/// A `UtcDateTime` represents a Unix timestamp on the UTC Calendar.
///
/// It can represent any date between `1-1-1 00:00:00` and `[u32::MAX]-12-31 23:59:59` (inclusive).
///
/// In terms of indexing:
/// * Months and days of month are 1-based (i.e. `Dec 15th 2022` corresponds to `2022-12-15`).
/// * Hour, minute and second are 0-based, based on the 24-hour clock.
///   Midnight is represented as `00:00:00` and `23:59:59` is the last second before midnight.
///   Following Unix timstamp conventions, leap seconds are not supported.
///
/// `UtcDateTime` supports methods for easy conversion to and from the [`Instant`](super::Instant) type, which
/// can be queried from the Radix Engine.
#[derive(Sbor, PartialEq, Eq, Copy, Clone, Debug)]
pub struct UtcDateTime {
    year: u32,
    month: u8,
    day_of_month: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl UtcDateTime {
    /// Creates a `UtcDateTime` from its individual components.
    ///
    /// Months and days of month are 1-based, with the following limits:
    /// * Valid year range is: `1` to `u32::MAX`inclusive.
    /// * Valid month values are: `1` to `12` inclusive.
    /// * Valid day of month values are: `1` to `{28, 29, 30, 31}` inclusive.
    ///   The upper limit depends on the month and year, as per the Gregorian calendar.
    ///
    /// An attempt to create an invalid date (e.g. `2022-04-31` or `2023-02-29`)
    /// will result in a corresponding `Err(DateTimeError)`.
    ///
    /// Hour, minute and second constitute a 0-based 24-hour clock.
    /// Midnight is represented as `00:00:00` and `23:59:59` is the maximum possible time (a second before midnight).
    /// Following Unix time conventions, leap seconds are not represented in this system.
    pub fn new(
        year: u32,
        month: u8,
        day_of_month: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, DateTimeError> {
        if year <= 0 {
            return Err(DateTimeError::InvalidYear);
        }

        if month < 1 || month > 12 {
            return Err(DateTimeError::InvalidMonth);
        }

        if day_of_month < 1 ||
            // Check leap year Feb + all other months
            day_of_month > LEAP_YEAR_DAYS_IN_MONTHS[(month - 1) as usize] ||
            // Check Feb on non-leap years
            (!Self::is_leap_year(year) && month == 2 && day_of_month > 28)
        {
            return Err(DateTimeError::InvalidDayOfMonth);
        }

        if hour > 23 {
            return Err(DateTimeError::InvalidHour);
        }

        if minute > 59 {
            return Err(DateTimeError::InvalidMinute);
        }

        if second > 59 {
            return Err(DateTimeError::InvalidSecond);
        }

        Ok(Self {
            year,
            month,
            day_of_month,
            hour,
            minute,
            second,
        })
    }

    /// Creates a `UtcDateTime` from an [`Instant`](super::Instant).
    ///
    /// The minimum supported `seconds_since_unix_epoch` value is `-62135596800` (corresponding to `1-1-1 00:00:00`)
    /// and the maximum value is `135536014634284799` (corresponding to `[u32::Max]-12-31 23:59:59`).
    pub fn from_instant(instant: &Instant) -> Result<Self, DateTimeError> {
        if instant.seconds_since_unix_epoch < MIN_SUPPORTED_TIMESTAMP
            || instant.seconds_since_unix_epoch > MAX_SUPPORTED_TIMESTAMP
        {
            return Err(DateTimeError::InstantIsOutOfRange);
        }

        // First, convert the base to 1 Mar 2000 for easier leap year calculation
        let secs_since_march_y2k =
            instant.seconds_since_unix_epoch - SHIFT_FROM_UNIX_TIME_TO_MARCH_Y2K;

        let mut days_since_march_y2k = secs_since_march_y2k / SECONDS_IN_A_DAY;
        let mut remaining_secs = secs_since_march_y2k % SECONDS_IN_A_DAY;
        if remaining_secs < 0 {
            remaining_secs += SECONDS_IN_A_DAY;
            days_since_march_y2k -= 1;
        }

        let mut num_400_year_cycles = days_since_march_y2k / DAYS_PER_400Y;
        let mut remaining_days = days_since_march_y2k % DAYS_PER_400Y;
        if remaining_days < 0 {
            remaining_days += DAYS_PER_400Y;
            num_400_year_cycles -= 1;
        }

        let mut num_100_year_cycles = remaining_days / DAYS_PER_100Y;
        if num_100_year_cycles == 4 {
            // Subtract one due 400 years cycle (400 years cycle fits 4 100y cycles)
            num_100_year_cycles -= 1;
        }
        remaining_days -= num_100_year_cycles * DAYS_PER_100Y;

        let mut num_4_year_cycles = remaining_days / DAYS_PER_4Y;
        if num_4_year_cycles == 25 {
            // Subtract one due 100 years cycle (100 years cycle fits 25 4y cycles)
            num_4_year_cycles -= 1;
        }
        remaining_days -= num_4_year_cycles * DAYS_PER_4Y;

        let mut remaining_years = remaining_days / 365;
        if remaining_years == 4 {
            // Subtract one due to four years cycle
            remaining_years -= 1;
        }
        remaining_days -= remaining_years * 365;

        let mut year =
            remaining_years +
                4 * num_4_year_cycles +
                100 * num_100_year_cycles +
                400 * num_400_year_cycles
                + 2000 /* Add the base year (after shifting) */;

        let mut days_in_months_starting_on_march = LEAP_YEAR_DAYS_IN_MONTHS.clone();
        days_in_months_starting_on_march.rotate_left(2);

        let mut month = 0;
        while days_in_months_starting_on_march[month] as i64 <= remaining_days {
            remaining_days -= days_in_months_starting_on_march[month] as i64;
            month += 1;
        }

        // Shift the month back to Jan
        // Handle any overflows, in case we need to add another year after shifting
        month += 2;
        if month >= 12 {
            month -= 12;
            year += 1;
        }

        // Shift 0-based month to 1-based
        month += 1;

        // Shift 0-based day of month to 1-based
        let day_of_month = remaining_days + 1;

        let hour = remaining_secs / SECONDS_IN_AN_HOUR;
        let minute = remaining_secs / SECONDS_IN_A_MINUTE % SECONDS_IN_A_MINUTE;
        let second = remaining_secs % SECONDS_IN_A_MINUTE;

        Ok(Self {
            year: u32::try_from(year).expect("year overflow"),
            month: u8::try_from(month).expect("month overflow"),
            day_of_month: u8::try_from(day_of_month).expect("day_of_month overflow"),
            hour: u8::try_from(hour).expect("hour overflow"),
            minute: u8::try_from(minute).expect("minute overflow"),
            second: u8::try_from(second).expect("second overflow"),
        })
    }

    /// Creates an [`Instant`](super::Instant) from this `UtcDateTime`
    pub fn to_instant(&self) -> Instant {
        let is_leap_year = Self::is_leap_year(self.year);

        // Separating pre-1970 (negative) and 1970 onward (non-negative)
        // timestamps for better readability
        if self.year >= UNIX_EPOCH_YEAR {
            // Count ended leap and non-leap years between Unix epoch and dt
            let num_leap_years_between_self_and_epoch =
                (Self::num_leap_years_up_to_exclusive(self.year)
                    - Self::num_leap_years_up_to_exclusive(UNIX_EPOCH_YEAR + 1))
                    as i64;

            let num_non_leap_years_between_self_and_epoch =
                (self.year - UNIX_EPOCH_YEAR) as i64 - num_leap_years_between_self_and_epoch;

            // Given the number of ended leap and non-leap years, count the elapsed seconds
            let seconds_up_to_the_beginning_of_the_year = (num_non_leap_years_between_self_and_epoch
                * SECONDS_IN_A_NON_LEAP_YEAR)
                + (num_leap_years_between_self_and_epoch * SECONDS_IN_A_LEAP_YEAR);

            // Count the seconds for ended months
            let mut seconds_in_ended_months = 0;
            for n in 0..self.month - 1 {
                seconds_in_ended_months +=
                    LEAP_YEAR_DAYS_IN_MONTHS[n as usize] as i64 * SECONDS_IN_A_DAY;
                // Subtract one day for any non-leap Feb
                if !is_leap_year && n == 1 {
                    seconds_in_ended_months -= SECONDS_IN_A_DAY;
                }
            }

            // Sum it all together and add remaining days, hours, minutes and seconds
            let total_seconds_since_unix_epoch = seconds_up_to_the_beginning_of_the_year
                + seconds_in_ended_months
                + (self.day_of_month - 1) as i64 * SECONDS_IN_A_DAY
                + self.hour as i64 * SECONDS_IN_AN_HOUR
                + self.minute as i64 * SECONDS_IN_A_MINUTE
                + self.second as i64;

            Instant::new(total_seconds_since_unix_epoch)
        } else {
            // Similarly, count the number of leap and non-leap years...
            let num_leap_years_between_epoch_and_self =
                (Self::num_leap_years_up_to_exclusive(UNIX_EPOCH_YEAR)
                    - Self::num_leap_years_up_to_exclusive(self.year + 1)) as i64;

            let num_non_leap_days_between_epoch_and_self =
                (UNIX_EPOCH_YEAR - self.year - 1) as i64 - num_leap_years_between_epoch_and_self;

            // ...and use it to count the number of seconds up (down?) to the end of year,
            // remember, we're counting backwards!
            let seconds_up_to_the_end_of_the_year = (num_non_leap_days_between_epoch_and_self
                * SECONDS_IN_A_NON_LEAP_YEAR)
                + (num_leap_years_between_epoch_and_self * SECONDS_IN_A_LEAP_YEAR);

            // We're counting backwards so add seconds for any non-started months
            let mut seconds_in_non_started_months = 0;
            let mut curr_month = 11;
            while curr_month > self.month - 1 {
                seconds_in_non_started_months +=
                    LEAP_YEAR_DAYS_IN_MONTHS[curr_month as usize] as i64 * SECONDS_IN_A_DAY;
                // Subtract one day for any non-leap Feb
                if !is_leap_year && curr_month == 1 {
                    seconds_in_non_started_months -= SECONDS_IN_A_DAY;
                }
                curr_month -= 1;
            }

            let mut days_in_month = LEAP_YEAR_DAYS_IN_MONTHS[self.month as usize - 1] as i64;
            if !is_leap_year && curr_month == 1 {
                days_in_month -= 1;
            }

            // Add the remaining days of the current month
            let remaining_days_in_month = days_in_month - self.day_of_month as i64;

            let total_seconds_since_unix_epoch = seconds_up_to_the_end_of_the_year
                + seconds_in_non_started_months
                + remaining_days_in_month * SECONDS_IN_A_DAY
                + (23 - self.hour) as i64 * SECONDS_IN_AN_HOUR
                + (59 - self.minute) as i64 * SECONDS_IN_A_MINUTE
                + (59 - self.second) as i64;

            Instant::new(
                // Pre-1970 timestamps are negative
                -total_seconds_since_unix_epoch - 1,
            )
        }
    }

    fn num_leap_years_up_to_exclusive(year: u32) -> u32 {
        let prev = year - 1;
        (prev / 4) - (prev / 100) + (prev / 400)
    }

    fn is_leap_year(year: u32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn year(&self) -> u32 {
        self.year
    }

    pub fn month(&self) -> u8 {
        self.month
    }

    pub fn day_of_month(&self) -> u8 {
        self.day_of_month
    }

    pub fn hour(&self) -> u8 {
        self.hour
    }

    pub fn minute(&self) -> u8 {
        self.minute
    }

    pub fn second(&self) -> u8 {
        self.second
    }

    pub fn add_days(&self, days_to_add: i64) -> Option<UtcDateTime> {
        self.to_instant()
            .add_days(days_to_add)
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_hours(&self, hours_to_add: i64) -> Option<UtcDateTime> {
        self.to_instant()
            .add_hours(hours_to_add)
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_minutes(&self, minutes_to_add: i64) -> Option<UtcDateTime> {
        self.to_instant()
            .add_minutes(minutes_to_add)
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_seconds(&self, seconds_to_add: i64) -> Option<UtcDateTime> {
        self.to_instant()
            .add_seconds(seconds_to_add)
            .and_then(|i| Self::from_instant(&i).ok())
    }
}

impl TryFrom<Instant> for UtcDateTime {
    type Error = DateTimeError;
    fn try_from(instant: Instant) -> Result<Self, Self::Error> {
        UtcDateTime::from_instant(&instant)
    }
}

impl From<UtcDateTime> for Instant {
    fn from(dt: UtcDateTime) -> Self {
        (&dt).to_instant()
    }
}

impl Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            self.year, self.month, self.day_of_month, self.hour, self.minute, self.second,
        )
    }
}

#[derive(Debug, Clone)]
pub enum ParseUtcDateTimeError {
    InvalidFormat,
    DateTimeError(DateTimeError),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseUtcDateTimeError {}

impl fmt::Display for ParseUtcDateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseUtcDateTimeError::InvalidFormat => write!(f, "Invalid date time format. Must be in ISO-8601 format, up to second precision, such as '2011-12-03T10:15:30Z'."),
            ParseUtcDateTimeError::DateTimeError(e) => e.fmt(f)
        }
    }
}

impl From<ParseIntError> for ParseUtcDateTimeError {
    fn from(_value: ParseIntError) -> Self {
        Self::InvalidFormat
    }
}

impl From<DateTimeError> for ParseUtcDateTimeError {
    fn from(value: DateTimeError) -> Self {
        Self::DateTimeError(value)
    }
}

impl FromStr for UtcDateTime {
    type Err = ParseUtcDateTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().into_iter().collect();
        if chars.len() == 20
            && chars[4] == '-'
            && chars[7] == '-'
            && chars[10] == 'T'
            && chars[13] == ':'
            && chars[16] == ':'
            && chars[19] == 'Z'
        {
            Ok(UtcDateTime::new(
                s[0..4].parse::<u32>()?,
                s[5..7].parse::<u8>()?,
                s[8..10].parse::<u8>()?,
                s[11..13].parse::<u8>()?,
                s[14..16].parse::<u8>()?,
                s[17..19].parse::<u8>()?,
            )?)
        } else {
            Err(ParseUtcDateTimeError::InvalidFormat)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time::utc_date_time::MAX_SUPPORTED_TIMESTAMP;
    use radix_engine_common::time::utc_date_time::MIN_SUPPORTED_TIMESTAMP;

    #[test]
    pub fn test_to_string() {
        let expected_str = "2023-01-27T12:17:25Z";
        let instant = Instant {
            seconds_since_unix_epoch: 1674821845,
        };
        let date_time = UtcDateTime::from_instant(&instant).unwrap();
        assert_eq!(date_time.to_string(), expected_str);
        assert_eq!(format!("{}", date_time), expected_str);
        assert_eq!(
            UtcDateTime::from_str(expected_str).unwrap().to_instant(),
            instant
        );
    }

    #[test]
    pub fn test_instant_date_time_conversions() {
        let test_data = vec![
            (MIN_SUPPORTED_TIMESTAMP, [1, 1, 1, 0, 0, 0]),
            (-62104060800, [2, 1, 1, 0, 0, 0]),
            (-62035804801, [4, 2, 29, 23, 59, 59]),
            (-30578688000, [1001, 1, 1, 0, 0, 0]),
            (-5233420801, [1804, 2, 28, 23, 59, 59]),
            (-2147483648, [1901, 12, 13, 20, 45, 52]),
            (-58147200, [1968, 2, 28, 00, 00, 00]),
            (-58147199, [1968, 2, 28, 00, 00, 01]),
            (-58060801, [1968, 2, 28, 23, 59, 59]),
            (-58060800, [1968, 2, 29, 00, 00, 00]),
            (-1, [1969, 12, 31, 23, 59, 59]),
            (0, [1970, 1, 1, 0, 0, 0]),
            (1, [1970, 1, 1, 0, 0, 1]),
            (365 * 24 * 60 * 60, [1971, 1, 1, 0, 0, 0]),
            (366 * 24 * 60 * 60, [1971, 1, 2, 0, 0, 0]),
            (395 * 24 * 60 * 60, [1971, 1, 31, 0, 0, 0]),
            (396 * 24 * 60 * 60, [1971, 2, 1, 0, 0, 0]),
            (68180521, [1972, 2, 29, 3, 2, 1]),
            (194476271, [1976, 2, 29, 21, 11, 11]),
            (446947199, [1984, 2, 29, 23, 59, 59]),
            (447012859, [1984, 3, 1, 18, 14, 19]),
            (951865200, [2000, 2, 29, 23, 0, 0]),
            (951868800, [2000, 3, 1, 0, 0, 0]),
            (1109548800, [2005, 2, 28, 0, 0, 0]),
            (1670420819, [2022, 12, 7, 13, 46, 59]),
            (1835395199, [2028, 2, 28, 23, 59, 59]),
            (1835395200, [2028, 2, 29, 00, 00, 00]),
            (1835481599, [2028, 2, 29, 23, 59, 59]),
            (1835481600, [2028, 3, 1, 00, 00, 00]),
            (51442991999, [3600, 2, 29, 23, 59, 59]),
            (64065686400, [4000, 2, 29, 0, 0, 0]),
            (569034205384, [20001, 12, 23, 1, 3, 4]),
            (MAX_SUPPORTED_TIMESTAMP, [u32::MAX, 12, 31, 23, 59, 59]),
        ];

        for (timestamp, dt_components) in test_data {
            let expected_dt = UtcDateTime::from(dt_components);
            let expected_instant = Instant::new(timestamp);

            assert_eq!(expected_dt.to_instant(), expected_instant);
            assert_eq!(
                UtcDateTime::from_instant(&expected_instant).unwrap(),
                expected_dt
            );
        }

        // Some error assertions (no unexpected panics)
        assert!(UtcDateTime::from_instant(&Instant::new(MAX_SUPPORTED_TIMESTAMP + 1)).is_err());
        assert!(UtcDateTime::from_instant(&Instant::new(MIN_SUPPORTED_TIMESTAMP - 1)).is_err());
        assert!(UtcDateTime::from_instant(&Instant::new(i64::MIN)).is_err());
        assert!(UtcDateTime::from_instant(&Instant::new(i64::MAX)).is_err());
    }

    #[test]
    pub fn test_date_time_add_xyz_methods() {
        assert_dates(
            [2022, 1, 1, 12, 12, 12],
            |dt| dt.add_days(2),
            [2022, 1, 3, 12, 12, 12],
        );

        assert_dates(
            [1968, 2, 29, 00, 00, 00],
            |dt| dt.add_days(2).and_then(|dt| dt.add_hours(2)),
            [1968, 3, 2, 02, 00, 00],
        );

        assert_dates(
            [2028, 2, 29, 23, 59, 59],
            |dt| dt.add_hours(49).and_then(|dt| dt.add_seconds(1)),
            [2028, 3, 3, 01, 00, 00],
        );

        assert_dates(
            [2022, 1, 1, 12, 12, 12],
            |dt| dt.add_days(2),
            [2022, 1, 3, 12, 12, 12],
        );

        assert_dates(
            [1, 1, 1, 0, 0, 0],
            |dt| dt.add_minutes(1000 * 365 * 23 * 60),
            [958, 9, 12, 16, 0, 0],
        );

        assert_dates(
            [1970, 1, 1, 0, 0, 0],
            |dt| dt.add_days(365),
            [1971, 1, 1, 0, 0, 0],
        );

        assert_dates(
            [1971, 1, 1, 0, 0, 0],
            |dt| dt.add_days(-365),
            [1970, 1, 1, 0, 0, 0],
        );

        assert_dates(
            [1968, 3, 1, 00, 00, 00],
            |dt| dt.add_seconds(-1),
            [1968, 2, 29, 23, 59, 59],
        );

        assert_fails([u32::MAX, 12, 31, 00, 00, 00], |dt| dt.add_days(1));
        assert_fails([u32::MAX, 12, 31, 23, 59, 59], |dt| dt.add_hours(1));
        assert_fails([u32::MAX, 12, 31, 23, 59, 59], |dt| dt.add_minutes(1));
        assert_fails([u32::MAX, 12, 31, 23, 59, 59], |dt| dt.add_seconds(1));

        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_days(i64::MAX));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_hours(i64::MAX));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_minutes(i64::MAX));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_seconds(i64::MAX));

        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_days(i64::MIN));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_hours(i64::MIN));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_minutes(i64::MIN));
        assert_fails([2000, 12, 31, 23, 59, 59], |dt| dt.add_seconds(i64::MIN));
    }

    fn assert_dates<F: FnOnce(UtcDateTime) -> Option<UtcDateTime>>(
        start: [u32; 6],
        op: F,
        expected_end: [u32; 6],
    ) {
        let start_dt = UtcDateTime::from(start);
        let expected_end_dt = UtcDateTime::from(expected_end);
        assert_eq!(op(start_dt).unwrap(), expected_end_dt);
    }

    fn assert_fails<F: FnOnce(UtcDateTime) -> Option<UtcDateTime>>(start: [u32; 6], op: F) {
        let start_dt = UtcDateTime::from(start);
        assert!(op(start_dt).is_none());
    }

    impl From<[u32; 6]> for UtcDateTime {
        fn from(dt: [u32; 6]) -> UtcDateTime {
            UtcDateTime::new(
                dt[0] as u32,
                dt[1] as u8,
                dt[2] as u8,
                dt[3] as u8,
                dt[4] as u8,
                dt[5] as u8,
            )
            .unwrap()
        }
    }
}
