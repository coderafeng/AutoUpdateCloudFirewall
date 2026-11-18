//! Sync scheduling: a fixed interval or a five-field cron expression.
//!
//! Every target follows the global settings unless it carries its own interval
//! or expression. A cron expression uses the classic five fields — minute,
//! hour, day of month, month, day of week — so `0 */5 * * *` means "every five
//! minutes" and `30 2 * * 1-5` means "02:30 on weekdays". The `@hourly`,
//! `@daily`, `@weekly`, `@monthly` and `@yearly` aliases are accepted too.
//!
//! Cron works on wall-clock time, intervals on elapsed time; the worker only
//! cares about "how long until the next run", which [`Schedule::delay`] gives.

use std::time::Duration;

use anyhow::{Result, bail};
use chrono::{
    DateTime, Datelike, Duration as ChronoDuration, Local, LocalResult, NaiveDate, NaiveDateTime,
    NaiveTime, TimeZone, Timelike,
};

use crate::model::{MIN_INTERVAL_SECS, Settings, Target};

/// A parsed five-field cron expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CronSchedule {
    minutes: Vec<bool>,
    hours: Vec<bool>,
    days: Vec<bool>,
    months: Vec<bool>,
    weekdays: Vec<bool>,
    day_any: bool,
    weekday_any: bool,
    expression: String,
}

impl CronSchedule {
    pub fn parse(expression: &str) -> Result<Self> {
        let raw = expression.trim();
        if raw.is_empty() {
            bail!("the expression is empty");
        }

        let expanded = match raw.to_ascii_lowercase().as_str() {
            "@hourly" => "0 * * * *",
            "@daily" | "@midnight" => "0 0 * * *",
            "@weekly" => "0 0 * * 0",
            "@monthly" => "0 0 1 * *",
            "@yearly" | "@annually" => "0 0 1 1 *",
            _ => raw,
        };

        let fields: Vec<&str> = expanded.split_whitespace().collect();
        if fields.len() != 5 {
            bail!(
                "expected 5 fields (minute hour day month weekday), found {}",
                fields.len()
            );
        }

        let mut weekdays = parse_field(fields[4], 0, 7)?;
        // Both 0 and 7 mean Sunday.
        if weekdays[7] {
            weekdays[0] = true;
            weekdays[7] = false;
        }

        Ok(Self {
            minutes: parse_field(fields[0], 0, 59)?,
            hours: parse_field(fields[1], 0, 23)?,
            days: parse_field(fields[2], 1, 31)?,
            months: parse_field(fields[3], 1, 12)?,
            weekdays,
            // A restricted day field combined with a restricted weekday field
            // matches when *either* matches, exactly like Vixie cron.
            day_any: fields[2].trim() == "*",
            weekday_any: fields[4].trim() == "*",
            expression: raw.to_string(),
        })
    }

    /// The expression as the user typed it.
    pub fn expression(&self) -> &str {
        &self.expression
    }

    /// The first matching minute strictly after `from`.
    pub fn next_after(&self, from: DateTime<Local>) -> Option<DateTime<Local>> {
        // Truncating to the minute means `from` itself is never returned.
        let start = (from + ChronoDuration::minutes(1))
            .with_second(0)?
            .with_nanosecond(0)?;
        let start_date = start.date_naive();
        let start_time = start.time();

        let mut day = start_date;
        for _ in 0..=366 * 8 {
            if self.day_matches(day) {
                let first = if day == start_date {
                    start_time
                } else {
                    NaiveTime::MIN
                };
                let mut hour = first.hour();
                while hour < 24 {
                    if self.hours[hour as usize] {
                        let first_minute = if hour == first.hour() {
                            first.minute()
                        } else {
                            0
                        };
                        for minute in first_minute..60 {
                            if !self.minutes[minute as usize] {
                                continue;
                            }
                            let naive =
                                NaiveDateTime::new(day, NaiveTime::from_hms_opt(hour, minute, 0)?);
                            match Local.from_local_datetime(&naive) {
                                LocalResult::Single(at) => return Some(at),
                                LocalResult::Ambiguous(at, _) => return Some(at),
                                // A DST gap simply has no such local minute.
                                LocalResult::None => {}
                            }
                        }
                    }
                    hour += 1;
                }
            }
            day = day.succ_opt()?;
        }
        None
    }

    fn day_matches(&self, day: NaiveDate) -> bool {
        if !self.months[day.month() as usize] {
            return false;
        }
        let dom = self.days[day.day() as usize];
        let dow = self.weekdays[day.weekday().num_days_from_sunday() as usize];
        match (self.day_any, self.weekday_any) {
            (true, true) => true,
            (true, false) => dow,
            (false, true) => dom,
            (false, false) => dom || dow,
        }
    }
}

/// `*`, `a`, `a-b`, `a-b/n`, `*/n` and comma-separated lists of those.
fn parse_field(spec: &str, low: u32, high: u32) -> Result<Vec<bool>> {
    let mut set = vec![false; (high + 1) as usize];
    for item in spec.split(',') {
        let item = item.trim();
        if item.is_empty() {
            bail!("`{spec}` contains an empty list item");
        }
        let (span, step) = match item.split_once('/') {
            Some((span, step)) => {
                let step: u32 = step
                    .trim()
                    .parse()
                    .map_err(|_| anyhow::anyhow!("`{}` is not a step", step.trim()))?;
                if step == 0 {
                    bail!("a step must be greater than zero");
                }
                (span.trim(), step)
            }
            None => (item, 1),
        };

        let (start, end) = if span == "*" {
            (low, high)
        } else if let Some((a, b)) = span.split_once('-') {
            let a = number(a, low, high)?;
            let b = number(b, low, high)?;
            if a > b {
                bail!("the range `{span}` is reversed");
            }
            (a, b)
        } else {
            let single = number(span, low, high)?;
            // `5/10` means "from 5 to the end, every 10".
            if step > 1 {
                (single, high)
            } else {
                (single, single)
            }
        };

        let mut value = start;
        while value <= end {
            set[value as usize] = true;
            value += step;
        }
    }
    if set.iter().all(|value| !*value) {
        bail!("`{spec}` selects nothing");
    }
    Ok(set)
}

fn number(text: &str, low: u32, high: u32) -> Result<u32> {
    let text = text.trim();
    let value: u32 = text
        .parse()
        .map_err(|_| anyhow::anyhow!("`{text}` is not a number"))?;
    if value < low || value > high {
        bail!("`{value}` is outside {low}-{high}");
    }
    Ok(value)
}

/// How a target is scheduled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Schedule {
    Interval(Duration),
    Cron(CronSchedule),
}

impl Schedule {
    /// Time from `now` until the next run.
    pub fn delay(&self, now: DateTime<Local>) -> Duration {
        match self {
            Schedule::Interval(duration) => *duration,
            Schedule::Cron(cron) => cron
                .next_after(now)
                .and_then(|next| (next - now).to_std().ok())
                .unwrap_or_else(|| Duration::from_secs(MIN_INTERVAL_SECS)),
        }
    }
}

/// The schedule used by targets that do not override it.
pub fn global(settings: &Settings) -> Schedule {
    resolve("", 0, &settings.schedule_cron, settings.check_interval_secs)
}

/// The schedule a single target follows.
pub fn for_target(settings: &Settings, target: &Target) -> Schedule {
    resolve(
        &target.schedule_cron,
        target.interval_secs,
        &settings.schedule_cron,
        settings.check_interval_secs,
    )
}

/// True when the target has no schedule of its own and the global one applies.
pub fn inherited(target: &Target) -> bool {
    target.schedule_cron.trim().is_empty() && target.interval_secs == 0
}

/// Target cron/interval first, then the global cron/interval.
fn resolve(target_cron: &str, target_secs: u64, global_cron: &str, global_secs: u64) -> Schedule {
    if let Ok(cron) = CronSchedule::parse(target_cron) {
        return Schedule::Cron(cron);
    }
    if target_secs > 0 {
        return Schedule::Interval(Duration::from_secs(target_secs.max(MIN_INTERVAL_SECS)));
    }
    if let Ok(cron) = CronSchedule::parse(global_cron) {
        return Schedule::Cron(cron);
    }
    Schedule::Interval(Duration::from_secs(global_secs.max(MIN_INTERVAL_SECS)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn at(hour: u32, minute: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(2026, 3, 10, hour, minute, 0)
            .single()
            .expect("valid local time")
    }

    fn next(expression: &str, from: DateTime<Local>) -> DateTime<Local> {
        CronSchedule::parse(expression)
            .expect("valid expression")
            .next_after(from)
            .expect("a next occurrence")
    }

    #[test]
    fn steps_advance_to_the_next_slot() {
        assert_eq!(next("*/5 * * * *", at(10, 2)), at(10, 5));
        assert_eq!(next("*/5 * * * *", at(10, 5)), at(10, 10));
        assert_eq!(next("*/5 * * * *", at(23, 58)).hour(), 0);
    }

    #[test]
    fn daily_and_weekly_expressions_cross_days() {
        let daily = next("0 3 * * *", at(4, 0));
        assert_eq!(daily, Local.with_ymd_and_hms(2026, 3, 11, 3, 0, 0).unwrap());

        // 2026-03-10 is a Tuesday, so the next Monday is the 16th.
        let monday = next("30 2 * * 1", at(9, 0));
        assert_eq!(
            monday,
            Local.with_ymd_and_hms(2026, 3, 16, 2, 30, 0).unwrap()
        );
    }

    #[test]
    fn aliases_and_month_boundaries_work() {
        assert_eq!(next("@hourly", at(10, 15)), at(11, 0));

        let first = next(
            "0 0 1 * *",
            Local.with_ymd_and_hms(2026, 3, 10, 8, 0, 0).unwrap(),
        );
        assert_eq!(first, Local.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap());
    }

    #[test]
    fn lists_ranges_and_steps_combine() {
        assert_eq!(next("0,30 * * * *", at(10, 1)), at(10, 30));
        assert_eq!(next("10-12 * * * *", at(10, 20)), at(11, 10));
        assert_eq!(next("5/15 * * * *", at(10, 6)), at(10, 20));
        // Sunday may be written as 0 or 7.
        let sunday = next(
            "0 0 * * 7",
            Local.with_ymd_and_hms(2026, 3, 10, 0, 0, 0).unwrap(),
        );
        assert_eq!(sunday.weekday(), chrono::Weekday::Sun);
    }

    #[test]
    fn invalid_expressions_are_rejected() {
        for bad in [
            "",
            "* * *",
            "* * * * * *",
            "60 * * * *",
            "*/0 * * * *",
            "5-1 * * * *",
            "a * * * *",
            "0 0 0 * *",
            "0 0 * 13 *",
            "0 0 * * 8",
        ] {
            assert!(
                CronSchedule::parse(bad).is_err(),
                "`{bad}` should not parse"
            );
        }
    }

    #[test]
    fn schedules_fall_back_to_the_global_settings() {
        let mut settings = Settings {
            check_interval_secs: 300,
            ..Settings::default()
        };

        let mut target = Target::new("acc", "tg");
        assert!(inherited(&target));
        assert_eq!(
            for_target(&settings, &target),
            Schedule::Interval(Duration::from_secs(300))
        );

        // A per-target interval wins over the global one and is clamped.
        target.interval_secs = 90;
        assert!(!inherited(&target));
        assert_eq!(
            for_target(&settings, &target),
            Schedule::Interval(Duration::from_secs(90))
        );
        target.interval_secs = 5;
        assert_eq!(
            for_target(&settings, &target),
            Schedule::Interval(Duration::from_secs(MIN_INTERVAL_SECS))
        );

        // A per-target expression wins over its own interval.
        target.interval_secs = 90;
        target.schedule_cron = "0 */10 * * *".to_string();
        assert!(matches!(for_target(&settings, &target), Schedule::Cron(_)));

        // The global expression applies when the target stays empty.
        settings.schedule_cron = "0 3 * * *".to_string();
        let follower = Target::new("acc", "tg2");
        assert!(matches!(
            for_target(&settings, &follower),
            Schedule::Cron(_)
        ));
        assert!(matches!(global(&settings), Schedule::Cron(_)));
    }

    #[test]
    fn delays_are_measurable() {
        let schedule = Schedule::Interval(Duration::from_secs(120));
        assert_eq!(schedule.delay(Local::now()), Duration::from_secs(120));

        let cron = Schedule::Cron(CronSchedule::parse("*/5 * * * *").unwrap());
        let delay = cron.delay(at(10, 2));
        assert_eq!(delay, Duration::from_secs(180));
    }
}
