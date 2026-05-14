use std::path::{Path, PathBuf};
use std::fs::OpenOptions;

use chrono::{Datelike, Local, NaiveDate};
use csv::{ReaderBuilder, WriterBuilder};

use crate::EventProvider;
use crate::events::{Category, Event, MonthDay, Rule};
use crate::filters::EventFilter;
use crate::providers::EventProviderError;

pub struct CSVFileProvider {
    name: String,
    path: PathBuf,
}

impl CSVFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

impl EventProvider for CSVFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(
        &self,
        filter: &EventFilter,
        events: &mut Vec<Event>,
    ) -> Result<(), EventProviderError> {
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(self.path.clone())
            .map_err(|error| EventProviderError::Io(format!("{}", error)))?;

        for result in reader.records() {
            let record = match result {
                Ok(record) => record,
                Err(error) => {
                    eprintln!("CSV record read error for '{}': {}", self.name, error);
                    continue;
                }
            };

            let mut date_string = record.get(0).unwrap_or("").to_string();
            let description = record.get(1).unwrap_or("").to_string();
            let category_string = record.get(2).unwrap_or("").to_string();

            let is_yearless = date_string.starts_with("--");
            if is_yearless {
                let today: NaiveDate = Local::now().date_naive();
                let year_string = format!("{:04}-", today.year());
                date_string = date_string.replace("--", &year_string);
            }

            let event: Event;

            if let Ok(date) = NaiveDate::parse_from_str(&date_string, "%F") {
                let category = Category::from_str(&category_string);
                if is_yearless {
                    event = Event::new_annual(
                        MonthDay::new(date.month(), date.day()),
                        description.clone(),
                        category,
                    );
                } else {
                    event = Event::new_singular(date, description.clone(), category);
                }
                if filter.accepts(&event) {
                    events.push(event);
                }
            } else if let Some(rule) = Rule::parse(&date_string) {
                let category = Category::from_str(&category_string);
                event = Event::new_rule_based(rule, description.clone(), category);
                if filter.accepts(&event) {
                    events.push(event);
                }
            } else {
                eprintln!("Invalid date '{}'", date_string);
            }
        }

        Ok(())
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        if !self.is_add_supported() {
            return Err(EventProviderError::OperationNotSupported);
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;

        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);

        let date = _event.date_string();
        let description = _event.description();
        let category = _event.category().to_string();

        writer
            .write_record(&[date, description, category])
            .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
        writer
            .flush()
            .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
        Ok(())
    }
}
