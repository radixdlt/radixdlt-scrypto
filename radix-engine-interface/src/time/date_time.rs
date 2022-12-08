use crate::time::constants::*;
use crate::time::Instant;
use sbor::*;
use sbor::rust::format;
use sbor::rust::string::{String, ToString};

const UNIX_EPOCH_YEAR: u32 = 1970;
const SECONDS_IN_A_NON_LEAP_YEAR: u64 = 365 * 24 * 60 * 60;
const SECONDS_IN_A_LEAP_YEAR: u64 = 366 * 24 * 60 * 60;
const DAYS_PER_4Y: i64 = 365 * 4 + 1;
const DAYS_PER_100Y: i64 = 365 * 100 + 24;
const DAYS_PER_400Y: i64 = 365 * 400 + 97;
const LEAP_YEAR_DAYS_IN_MONTHS: [u8; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
// A shift (in seconds) from the Unix epoch (1970-01-01 00:00:00)
// to a base date that is a multiple of a 400-year leap cycle.
// Used in Instant -> DateTime conversion.
// The date we're using is 2000-03-01 00:00:00, for two reasons:
// a) It's a multiple of 400, to make it easier to work with leap years
// b) We're also shifting the month to 1st March, so that
//    the extra day on leap years is added to the last month (Feb),
//    not in the middle of a year (makes some calculations easier)
const SHIFT_FROM_UNIX_TIME_TO_MARCH_Y2K: i64 = 946684800 + 86400 * (31 + 29);
const MIN_SUPPORTED_TIMESTAMP: i64 = -62135596800;
const MAX_SUPPORTED_TIMESTAMP: i64 = 135536014634284799;

#[derive(Encode, Decode, TypeId, PartialEq, Eq, Copy, Clone, Debug)]
pub struct DateTime {
    year: u32,
    month: u8,
    day_of_month: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

impl DateTime {
    pub fn new(
        year: u32,
        month: u8,
        day_of_month: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Self, String> {
        if year <= 0 {
            return Err("Invalid year. Expected a value strictly greater than 0".to_string());
        }

        if month < 1 || month > 12 {
            return Err(
                "Invalid month. Expected a value between 1 (inclusive) and 12 (inclusive)"
                    .to_string(),
            );
        }

        if day_of_month < 1 ||
            // Check leap year Feb + all other months
            day_of_month > LEAP_YEAR_DAYS_IN_MONTHS[(month - 1) as usize] ||
            // Check Feb on non-leap years
            (!Self::is_leap_year(year) && month == 2 && day_of_month > 28)
        {
            return Err("Invalid day of month.\
                Expected a value between 1 (inclusive)\
                and, depending on a month, 28, 29 (Feb on a leap year), 30 or 31 (inclusive)"
                .to_string());
        }

        if hour > 23 {
            return Err(
                "Invalid hour. Expected a value between 0 (inclusive) and 23 (inclusive)"
                    .to_string(),
            );
        }

        if minute > 59 {
            return Err(
                "Invalid minute. Expected a value between 0 (inclusive) and 59 (inclusive)"
                    .to_string(),
            );
        }

        if second > 59 {
            return Err(
                "Invalid second. Expected a value between 0 (inclusive) and 59 (inclusive)"
                    .to_string(),
            );
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

    pub fn from_instant(instant: &Instant) -> Result<Self, String> {
        if instant.seconds_since_unix_epoch < MIN_SUPPORTED_TIMESTAMP
            || instant.seconds_since_unix_epoch > MAX_SUPPORTED_TIMESTAMP
        {
            return Err(format!(
                "Instant out of supported range [{}, {}]",
                MIN_SUPPORTED_TIMESTAMP, MAX_SUPPORTED_TIMESTAMP
            ));
        }

        // First, convert the base to 1 Mar 2000 for easier leap year calculation
        let secs_since_march_y2k: i64 =
            instant.seconds_since_unix_epoch - SHIFT_FROM_UNIX_TIME_TO_MARCH_Y2K;

        let mut days_since_march_y2k = secs_since_march_y2k / SECONDS_IN_A_DAY as i64;
        let mut remaining_secs = secs_since_march_y2k % SECONDS_IN_A_DAY as i64;
        if remaining_secs < 0 {
            remaining_secs += SECONDS_IN_A_DAY as i64;
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

        let hour = remaining_secs / SECONDS_IN_AN_HOUR as i64;
        let minute = remaining_secs / SECONDS_IN_A_MINUTE as i64 % SECONDS_IN_A_MINUTE as i64;
        let second = remaining_secs % SECONDS_IN_A_MINUTE as i64;

        Ok(Self {
            year: u32::try_from(year).map_err(|_| "year overflow")?,
            month: u8::try_from(month).map_err(|_| "month overflow")?,
            day_of_month: u8::try_from(day_of_month).map_err(|_| "day_of_month overflow")?,
            hour: u8::try_from(hour).map_err(|_| "hour overflow")?,
            minute: u8::try_from(minute).map_err(|_| "minute overflow")?,
            second: u8::try_from(second).map_err(|_| "second overflow")?,
        })
    }

    pub fn to_instant(&self) -> Result<Instant, String> {
        let is_leap_year = Self::is_leap_year(self.year);

        // Separating pre-1970 (negative) and 1970 onward (non-negative)
        // timestamps for better readability
        if self.year >= UNIX_EPOCH_YEAR {
            // Count ended leap and non-leap years between Unix epoch and dt
            let num_leap_years_between_self_and_epoch =
                Self::num_leap_years_up_to_exclusive(self.year)
                    - Self::num_leap_years_up_to_exclusive(UNIX_EPOCH_YEAR + 1);

            let num_non_leap_years_between_self_and_epoch =
                (self.year - UNIX_EPOCH_YEAR) - num_leap_years_between_self_and_epoch;

            // Given the number of ended leap and non-leap years, count the elapsed seconds
            let seconds_up_to_the_beginning_of_the_year =
                (num_non_leap_years_between_self_and_epoch as u64 * SECONDS_IN_A_NON_LEAP_YEAR)
                    + (num_leap_years_between_self_and_epoch as u64 * SECONDS_IN_A_LEAP_YEAR);

            // Count the seconds for ended months
            let mut seconds_in_ended_months: u64 = 0;
            for n in 0..self.month - 1 {
                seconds_in_ended_months +=
                    LEAP_YEAR_DAYS_IN_MONTHS[n as usize] as u64 * SECONDS_IN_A_DAY as u64;
                // Subtract one day for any non-leap Feb
                if !is_leap_year && n == 1 {
                    seconds_in_ended_months -= SECONDS_IN_A_DAY as u64;
                }
            }

            // Sum it all together and add remaining days, hours, minutes and seconds
            let total_seconds_since_unix_epoch = seconds_up_to_the_beginning_of_the_year
                + seconds_in_ended_months
                + (self.day_of_month - 1) as u64 * SECONDS_IN_A_DAY as u64
                + self.hour as u64 * SECONDS_IN_AN_HOUR as u64
                + self.minute as u64 * SECONDS_IN_A_MINUTE as u64
                + self.second as u64;

            Ok(Instant::new(
                total_seconds_since_unix_epoch as i64, /* guaranteed to fit in i64 */
            ))
        } else {
            // Similarly, count the number of leap and non-leap years...
            let num_leap_years_between_epoch_and_self =
                Self::num_leap_years_up_to_exclusive(UNIX_EPOCH_YEAR)
                    - Self::num_leap_years_up_to_exclusive(self.year + 1);

            let num_non_leap_days_between_epoch_and_self =
                (UNIX_EPOCH_YEAR - self.year - 1) - num_leap_years_between_epoch_and_self;

            // ...and use it to count the number of seconds up (down?) to the end of year,
            // remember, we're counting backwards!
            let seconds_up_to_the_end_of_the_year =
                (num_non_leap_days_between_epoch_and_self as u64 * SECONDS_IN_A_NON_LEAP_YEAR)
                    + (num_leap_years_between_epoch_and_self as u64 * SECONDS_IN_A_LEAP_YEAR);

            // We're counting backwards so add seconds for any non-started months
            let mut seconds_in_non_started_months: u64 = 0;
            let mut curr_month = 11;
            while curr_month > self.month - 1 {
                seconds_in_non_started_months +=
                    LEAP_YEAR_DAYS_IN_MONTHS[curr_month as usize] as u64 * SECONDS_IN_A_DAY as u64;
                // Subtract one day for any non-leap Feb
                if !is_leap_year && curr_month == 1 {
                    seconds_in_non_started_months -= SECONDS_IN_A_DAY as u64;
                }
                curr_month -= 1;
            }

            let mut days_in_month = LEAP_YEAR_DAYS_IN_MONTHS[self.month as usize - 1];
            if !is_leap_year && curr_month == 1 {
                days_in_month -= 1;
            }

            // Add the remaining days of the current month
            let remaining_days_in_month = days_in_month - self.day_of_month;

            let total_seconds_since_unix_epoch = seconds_up_to_the_end_of_the_year
                + seconds_in_non_started_months
                + remaining_days_in_month as u64 * SECONDS_IN_A_DAY as u64
                + (23 - self.hour) as u64 * SECONDS_IN_AN_HOUR as u64
                + (59 - self.minute) as u64 * SECONDS_IN_A_MINUTE as u64
                + (59 - self.second) as u64;

            Ok(Instant::new(
                // Pre-1970 timestamps are negative
                -(total_seconds_since_unix_epoch as i64) - 1,
            ))
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

    pub fn add_days(&self, days_to_add: i64) -> Option<DateTime> {
        self.to_instant()
            .ok()
            .and_then(|i| i.add_days(days_to_add))
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_hours(&self, hours_to_add: i64) -> Option<DateTime> {
        self.to_instant()
            .ok()
            .and_then(|i| i.add_hours(hours_to_add))
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_minutes(&self, minutes_to_add: i64) -> Option<DateTime> {
        self.to_instant()
            .ok()
            .and_then(|i| i.add_minutes(minutes_to_add))
            .and_then(|i| Self::from_instant(&i).ok())
    }

    pub fn add_seconds(&self, seconds_to_add: i64) -> Option<DateTime> {
        self.to_instant()
            .ok()
            .and_then(|i| i.add_seconds(seconds_to_add))
            .and_then(|i| Self::from_instant(&i).ok())
    }
}

impl TryFrom<Instant> for DateTime {
    type Error = String;
    fn try_from(instant: Instant) -> Result<Self, Self::Error> {
        DateTime::from_instant(&instant)
    }
}

impl TryFrom<DateTime> for Instant {
    type Error = String;
    fn try_from(dt: DateTime) -> Result<Self, Self::Error> {
        (&dt).to_instant()
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTime, Instant};
    use crate::time::date_time::MAX_SUPPORTED_TIMESTAMP;
    use radix_engine_interface::time::date_time::MIN_SUPPORTED_TIMESTAMP;

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
            (951868800, [2000, 3, 1, 0, 0, 0]),
            (1670420819, [2022, 12, 7, 13, 46, 59]),
            (1835395199, [2028, 2, 28, 23, 59, 59]),
            (1835395200, [2028, 2, 29, 00, 00, 00]),
            (1835481599, [2028, 2, 29, 23, 59, 59]),
            (1835481600, [2028, 3, 1, 00, 00, 00]),
            (51442991999, [3600, 2, 29, 23, 59, 59]),
            (569034205384, [20001, 12, 23, 1, 3, 4]),
            (MAX_SUPPORTED_TIMESTAMP, [u32::MAX, 12, 31, 23, 59, 59]),
        ];

        for (timestamp, dt_components) in test_data {
            let expected_dt = DateTime::from(dt_components);
            let expected_instant = Instant::new(timestamp);

            assert_eq!(expected_dt.to_instant().unwrap(), expected_instant);
            assert_eq!(
                DateTime::from_instant(&expected_instant).unwrap(),
                expected_dt
            );
        }

        // Some error assertions (no unexpected panics)
        assert!(DateTime::from_instant(&Instant::new(MAX_SUPPORTED_TIMESTAMP + 1)).is_err());
        assert!(DateTime::from_instant(&Instant::new(MIN_SUPPORTED_TIMESTAMP - 1)).is_err());
        assert!(DateTime::from_instant(&Instant::new(i64::MIN)).is_err());
        assert!(DateTime::from_instant(&Instant::new(i64::MAX)).is_err());
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

    fn assert_dates<F: FnOnce(DateTime) -> Option<DateTime>>(
        start: [u32; 6],
        op: F,
        expected_end: [u32; 6],
    ) {
        let start_dt = DateTime::from(start);
        let expected_end_dt = DateTime::from(expected_end);
        assert_eq!(op(start_dt).unwrap(), expected_end_dt);
    }

    fn assert_fails<F: FnOnce(DateTime) -> Option<DateTime>>(start: [u32; 6], op: F) {
        let start_dt = DateTime::from(start);
        assert!(op(start_dt).is_none());
    }

    impl From<[u32; 6]> for DateTime {
        fn from(dt: [u32; 6]) -> DateTime {
            DateTime::new(
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
