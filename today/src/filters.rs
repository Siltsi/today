use std::collections::HashSet;

use crate::events::{Category, Event, MonthDay};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FilterOption {
    MonthDay(MonthDay),
    Category(Category),
    Text(String),
    Exclude(Category),
}

#[derive(Debug, PartialEq)]
pub struct EventFilter {
    options: HashSet<FilterOption>,
}

impl EventFilter {
    pub fn new() -> Self {
        Self {
            options: HashSet::new(),
        }
    }

    pub fn accepts(&self, event: &Event) -> bool {
        // If the option set is empty, this is an all-pass filter.
        if self.options.is_empty() {
            return true;
        }

        // Collect the results from various options into a vector.
        let mut results: Vec<bool> = Vec::new();

        for option in self.options.iter() {
            let result = match option {
                FilterOption::MonthDay(month_day) => {
                    *month_day == event.month_day()
                },
                FilterOption::Category(category) => {
                    *category == event.category()
                },
                FilterOption::Text(text) => {
                    event.description().contains(text)
                },
                FilterOption::Exclude(category) => {
                    *category != event.category()
                },
            };
            results.push(result);
        }
        // If the results vector contains only `true`` values,
        // all the options match, and the event will be accepted,
        // otherwise it will be rejected by the filter.
        results.iter().all(|&option| option)
    }

    pub fn contains_month_day(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::MonthDay(_)))
    }

    pub fn contains_category(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Category(_)))
    }

    pub fn contains_text(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Text(_)))
    }

    pub fn month_day(&self) -> Option<MonthDay> {
        for option in self.options.iter() {
            match option {
                FilterOption::MonthDay(month_day) => return Some(month_day.clone()),
                _ => (),
            }
        }
        // All checked, not found
        None
    }
    
    pub fn category(&self) -> Option<Category> {
        for option in self.options.iter() {
            match option {
                FilterOption::Category(category) => return Some(category.clone()),
                _ => (),
            }
        }
        None
    }
    
    pub fn text(&self) -> Option<String> {
        for option in self.options.iter() {
            match option {
                FilterOption::Text(text) => return Some(text.clone()),
                _ => (),
            }
        }
        None
    }
}

pub struct FilterBuilder {
    options: HashSet<FilterOption>,
}

impl FilterBuilder {
    pub fn new() -> FilterBuilder {
        FilterBuilder {
            options: HashSet::new(),
        }
    }

    pub fn contains_month_day(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::MonthDay(_)))
    }

    pub fn contains_category(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Category(_)))
    }

    pub fn contains_text(&self) -> bool {
        self.options
            .iter()
            .any(|option| matches!(option, &FilterOption::Text(_)))
    }

    pub fn month_day(mut self, month_day: MonthDay) -> FilterBuilder {
        if self.contains_month_day() {
            eprintln!("Month_day already set");
        } else {
            self.options.insert(FilterOption::MonthDay(month_day));
        }
        self
    }

    pub fn category(mut self, category: Category) -> FilterBuilder {
        if self.contains_category() {
            eprintln!("Category already set");
        } else {
            self.options.insert(FilterOption::Category(category));
        }
        self
    }

    pub fn text(mut self, text: String) -> FilterBuilder {
        if self.contains_text() {
            eprintln!("Text already set");
        } else {
            self.options.insert(FilterOption::Text(text));
        }
        self
    }

    pub fn exclude(mut self, category: Category) -> FilterBuilder {
        self.options.insert(FilterOption::Exclude(category));
        self
    }

    pub fn build(self) -> EventFilter {
        EventFilter {
            options: self.options,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn filter_accepts_anything() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );
        let filter = FilterBuilder::new().build();
        assert!(filter.accepts(&event));
    }

    #[test]
    fn build_filter_no_options() {
        let filter = FilterBuilder::new().build();
        let contains = [
            filter.contains_month_day(),
            filter.contains_category(),
            filter.contains_text(),
        ];
        assert_eq!(contains, [false, false, false]);
    }

    #[test]
    fn filter_accepts_month_day() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let month_day = MonthDay::new(3, 5);
        let filter = FilterBuilder::new()
            .month_day(month_day)
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_month_day() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let month_day = MonthDay::new(3, 6);
        let filter = FilterBuilder::new()
            .month_day(month_day)
            .build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_category() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let filter = FilterBuilder::new()
            .category(rust_category)
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_category() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let filter = FilterBuilder::new()
            .category(Category::new("programming", "python"))
            .build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_text() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let filter = FilterBuilder::new()
            .text("released".to_string())
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_text() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let filter = FilterBuilder::new()
            .text("updated".to_string())
            .build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn filter_accepts_all() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let month_day = MonthDay::new(3, 5);
        let filter = FilterBuilder::new()
            .month_day(month_day)
            .category(rust_category)
            .text("released".to_string())
            .build();

        assert!(filter.accepts(&event));
    }

    #[test]
    fn filter_rejects_some() {
        let rust_category = Category::new("programming", "rust");
        let event = Event::new_singular(
            NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            "Rust 1.94.0 released".to_string(),
            rust_category.clone(),
        );

        let month_day = MonthDay::new(3, 6);
        let filter = FilterBuilder::new()
            .month_day(month_day)
            .category(rust_category)
            .text("released".to_string())
            .build();

        assert!(!filter.accepts(&event));
    }

    #[test]
    fn duplicate_options_dropped() {
        let month_day = MonthDay::new(3, 5);
        let month_day2 = MonthDay::new(3, 6);

        let rust_category = Category::new("programming", "rust");
        let python_category = Category::new("programming", "python");

        let filter = FilterBuilder::new()
            .month_day(month_day)
            .month_day(month_day2)
            .category(rust_category)
            .category(python_category)
            .text("released".to_string())
            .text("updated".to_string())
            .build();

        assert_eq!(
            filter,
            FilterBuilder::new()
                .month_day(MonthDay::new(3, 5))
                .category(Category::new("programming", "rust"))
                .text("released".to_string())
                .build()
        )
    }
}
