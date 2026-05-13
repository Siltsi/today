use std::fmt;

use chrono::{Datelike, NaiveDate};

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
            return Err("Expexcted date in MM-DD format".to_string());
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
    pub fn month(&self) -> u32 { self.month }
    pub fn day(&self) -> u32 { self.day }
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

#[derive(Debug, PartialEq, Clone)]
pub enum EventKind {
    Singular(NaiveDate),
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
    #[allow(dead_code)]
    fn year(&self) -> i32 {
        match &self.kind {
            EventKind::Singular(date) => date.year(),
        }
    }
    pub fn month_day(&self) -> MonthDay {
        match &self.kind {
            EventKind::Singular(date) => MonthDay {
                month: date.month(),
                day: date.day(),
            },
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
            EventKind::Singular(date) => {
                date.format("%Y-%m-%d").to_string()
            }
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}: {} ({})",
            match &self.kind {
                EventKind::Singular(date) => date.year(),
            },
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