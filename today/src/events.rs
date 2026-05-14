use std::fmt;
use std::str::FromStr;

use chrono::{Datelike, Local, Month, NaiveDate, Weekday as ChronoWeekday};
use strum_macros::{Display, EnumString};

fn days_in_month(month: u32) -> u32 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 => 29,
        _ => 31,
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct MonthDay {
    month: u32,
    day: u32,
}

impl MonthDay {
    pub fn new(month: u32, day: u32) -> Self {
        Self { month, day }
    }
    pub fn from_str(s: &str) -> Result<Self, String> {
        if s.len() != 4 {
            return Err("MonthDay string must be exactly 4 characters in MMDD format".to_string());
        }

        let month = s[..2]
            .parse::<u32>()
            .map_err(|_| "Invalid month".to_string())?;
        if month == 0 || month > 12 {
            return Err("Invalid month".to_string());
        }

        let day = s[2..]
            .parse::<u32>()
            .map_err(|_| "Invalid day".to_string())?;
        if day == 0 || day > days_in_month(month) {
            return Err("Invalid day".to_string());
        }

        Ok(MonthDay { month, day })
    }
    pub fn from_str_split(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err("Expected date in MM-DD format".to_string());
        }
        let month = match parts[0].to_string().parse::<u32>() {
            Ok(m) => {
                if m > 12 {
                    return Err("Invalid month".to_string());
                }
                m
            }
            Err(_) => return Err("Invalid month".to_string()),
        };
        let day = match parts[1].to_string().parse::<u32>() {
            Ok(d) => {
                if d > days_in_month(month) {
                    return Err("Invalid day".to_string());
                }
                d
            }
            Err(_) => return Err("Invalid day".to_string()),
        };
        Ok(MonthDay { month, day })
    }
    pub fn month(&self) -> u32 {
        self.month
    }
    pub fn day(&self) -> u32 {
        self.day
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Category {
    primary: String,
    secondary: Option<String>,
}

impl Category {
    pub fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: Some(secondary.to_string()),
        }
    }
    pub fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: None,
        }
    }
    pub fn from_str(s: &str) -> Category {
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() < 2 {
            Category {
                primary: parts[0].to_string(),
                secondary: None,
            }
        } else {
            Category {
                primary: parts[0].to_string(),
                secondary: Some(parts[1].to_string()),
            }
        }
    }
    pub fn primary(&self) -> String {
        return self.primary.clone();
    }
    pub fn secondary(&self) -> String {
        match &self.secondary {
            Some(sec) => sec.clone(),
            None => "".to_string(),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.secondary {
            Some(sec) => write!(f, "{}/{}", self.primary, sec),
            None => write!(f, "{}", self.primary),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, EnumString, Display)]
#[strum(ascii_case_insensitive)]
pub enum Ordinal {
    First = 1,
    Second = 2,
    Third = 3,
    Fourth = 4,
    Last = 5,
}

#[derive(Debug, Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub enum Weekday {
    Monday = 0,
    Tuesday = 1,
    Wednesday = 2,
    Thursday = 3,
    Friday = 4,
    Saturday = 5,
    Sunday = 6,
}
impl Weekday {
    pub fn as_chrono_weekday(&self) -> ChronoWeekday {
        match *self {
            Weekday::Monday => ChronoWeekday::Mon,
            Weekday::Tuesday => ChronoWeekday::Tue,
            Weekday::Wednesday => ChronoWeekday::Wed,
            Weekday::Thursday => ChronoWeekday::Thu,
            Weekday::Friday => ChronoWeekday::Fri,
            Weekday::Saturday => ChronoWeekday::Sat,
            Weekday::Sunday => ChronoWeekday::Sun,
        }
    }
    pub fn from_chrono_weekday(wd: ChronoWeekday) -> Self {
        match wd {
            ChronoWeekday::Mon => Weekday::Monday,
            ChronoWeekday::Tue => Weekday::Tuesday,
            ChronoWeekday::Wed => Weekday::Wednesday,
            ChronoWeekday::Thu => Weekday::Thursday,
            ChronoWeekday::Fri => Weekday::Friday,
            ChronoWeekday::Sat => Weekday::Saturday,
            ChronoWeekday::Sun => Weekday::Sunday,
        }
    }
}

fn nth_weekday_in_month(
    year: i32,
    month: Month,
    weekday: Weekday,
    ordinal: Ordinal,
) -> Option<NaiveDate> {
    let mut count = 0;
    for day in 1..=31 {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month.number_from_month(), day) {
            if date.weekday() == weekday.as_chrono_weekday() {
                count += 1;
                if count == ordinal as i32 {
                    return Some(date);
                }
            }
        }
    }
    None
}

fn last_weekday_in_month(year: i32, month: Month, weekday: Weekday) -> Option<NaiveDate> {
    for day in (1..=31).rev() {
        // note that the range is reversed!
        if let Some(date) = NaiveDate::from_ymd_opt(year, month.number_from_month(), day) {
            if date.weekday() == weekday.as_chrono_weekday() {
                return Some(date);
            }
        }
    }
    None
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rule {
    ordinal: Ordinal,
    weekday: Weekday,
    month: Month,
}

impl Rule {
    // Parse a rule of the following format:
    // first|second|third|fourth|fifth|last <weekday> in <month>
    // weekday: Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday
    // month: January|February|March| ... |November|December
    pub fn parse(rule_string: &str) -> Option<Self> {
        let parts: Vec<String> = rule_string
            .to_lowercase()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        // After splitting on whitespace, there must be exactly four parts.
        if parts.len() != 4 {
            eprintln!("invalid rule: {}", rule_string);
            return None;
        }
        let ordinal = match Ordinal::from_str(&parts[0]) {
            Ok(ord) => ord,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        let weekday = match parts[1].parse::<ChronoWeekday>() {
            Ok(wd) => wd,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        if parts[2] != "in" && parts[2] != "of" {
            eprintln!("rule should specify `in` or `of`");
            return None;
        }
        let month = match parts[3].parse::<Month>() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                return None;
            }
        };
        Some(Self {
            ordinal,
            weekday: Weekday::from_chrono_weekday(weekday),
            month,
        })
    }

    pub fn resolve_date(&self, year: i32) -> Option<NaiveDate> {
        if self.ordinal == Ordinal::Last {
            last_weekday_in_month(year, self.month, self.weekday)
        } else {
            nth_weekday_in_month(year, self.month, self.weekday, self.ordinal)
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{} {} of {:?}",
            self.ordinal.to_string().to_lowercase(),
            self.weekday.as_chrono_weekday(),
            self.month
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EventKind {
    Singular(NaiveDate),
    Annual(MonthDay),
    RuleBased(Rule),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    kind: EventKind,
    description: String,
    category: Category,
}

impl Event {
    pub fn new_singular(date: NaiveDate, description: String, category: Category) -> Self {
        Event {
            kind: EventKind::Singular(date),
            description,
            category,
        }
    }

    pub fn new_annual(month_day: MonthDay, description: String, category: Category) -> Self {
        Self {
            kind: EventKind::Annual(month_day),
            description,
            category,
        }
    }

    pub fn new_rule_based(rule: Rule, description: String, category: Category) -> Self {
        Self {
            kind: EventKind::RuleBased(rule),
            description,
            category,
        }
    }

    pub fn year(&self) -> i32 {
        let today: NaiveDate = Local::now().date_naive();
        match &self.kind {
            EventKind::Singular(date) => date.year(),
            EventKind::Annual(_month_day) => today.year(),
            EventKind::RuleBased(_rule) => today.year(),
        }
    }

    pub fn month_day(&self) -> MonthDay {
        let today: NaiveDate = Local::now().date_naive();
        match &self.kind {
            EventKind::Singular(date) => MonthDay {
                month: date.month(),
                day: date.day(),
            },
            EventKind::Annual(month_day) => MonthDay {
                month: month_day.month,
                day: month_day.day,
            },
            EventKind::RuleBased(rule) => {
                let date = rule.resolve_date(today.year()).unwrap();

                MonthDay {
                    month: date.month(),
                    day: date.day(),
                }
            }
        }
    }

    pub fn category(&self) -> Category {
        self.category.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn kind(&self) -> EventKind {
        self.kind.clone()
    }

    pub fn date_string(&self) -> String {
        match &self.kind {
            EventKind::Singular(date) => date.format("%Y-%m-%d").to_string(),
            EventKind::Annual(md) => format!("--{}-{}", md.month, md.day),
            EventKind::RuleBased(rule) => rule.to_string(),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            self.year(),
            self.description,
            self.category
        )
    }
}

pub fn parse_excludes(excludes: &str) -> Vec<Category> {
    let mut categories: Vec<Category> = Vec::new();
    let parts: Vec<&str> = excludes.split(",").collect();
    for part in parts {
        categories.push(Category::from_str(part));
    }
    categories
}

pub fn params_to_event(date_string: &str, description_string: &str, category_string: &str) -> Result<Event, String> {
    let mut date_string = date_string.to_string();
    let is_yearless = date_string.starts_with("--");
    if is_yearless {
        let today: NaiveDate = Local::now().date_naive();
        let year_string = format!("{:04}-", today.year());
        date_string = date_string.replace("--", &year_string);
    }

    let event = if let Ok(date) = NaiveDate::parse_from_str(&date_string, "%F") {
        let category = Category::from_str(category_string);
        if is_yearless {
            Event::new_annual(
                MonthDay::new(date.month(), date.day()),
                description_string.to_string(),
                category,
            )
        } else {
            Event::new_singular(date, description_string.to_string(), category)
        }
    } else if let Some(rule) = Rule::parse(&date_string) {
        let category = Category::from_str(category_string);
        Event::new_rule_based(rule, description_string.to_string(), category)
    } else {
        return Err(format!("Invalid date '{}'", date_string));
    };

    Ok(event)
}
