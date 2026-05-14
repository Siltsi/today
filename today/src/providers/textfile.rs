use chrono::{Datelike, Local, NaiveDate};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::EventProvider;
use crate::events::{Category, Event, EventKind, MonthDay, Rule};
use crate::filters::EventFilter;
use crate::providers::EventProviderError;

enum ReadingState {
    Date,
    Description,
    Category,
    Separator,
}

pub struct TextFileProvider {
    name: String,
    path: PathBuf,
}

impl TextFileProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }
}

impl EventProvider for TextFileProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(
        &self,
        filter: &EventFilter,
        events: &mut Vec<Event>,
    ) -> Result<(), EventProviderError> {
        let f = File::open(self.path.clone())
            .map_err(|error| EventProviderError::Io(format!("{}", error)))?;
        let reader = BufReader::new(f);
        let mut state = ReadingState::Date;
        let mut date_string = String::new();
        let mut description = String::new();
        let mut category_string = String::new();

        for line_result in reader.lines() {
            let line = line_result.map_err(|error| EventProviderError::Io(format!("{}", error)))?;
            match state {
                ReadingState::Date => {
                    date_string = line;
                    state = ReadingState::Description;
                }
                ReadingState::Description => {
                    description = line;
                    state = ReadingState::Category;
                }
                ReadingState::Category => {
                    category_string = line;
                    state = ReadingState::Separator;
                }
                ReadingState::Separator => {
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

                    state = ReadingState::Date;
                }
            }
        }

        Ok(())
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, event: &Event) -> Result<(), EventProviderError> {
        if !self.is_add_supported() {
            return Err(EventProviderError::OperationNotSupported);
        }

        let file = OpenOptions::new()
            .append(true)
            .open(self.path.clone())
            .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
        let mut writer = BufWriter::new(file);
        
        return match event.kind() {
            EventKind::Singular(date) => {
                writeln!(writer, "{date}")
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.description())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.category())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer)
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                Ok(())
            }
            EventKind::Annual(_md) => {
                writeln!(writer, "{}", event.date_string())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.description())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.category())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer)
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                Ok(())
            }
            EventKind::RuleBased(_rule) => {
                writeln!(writer, "{}", event.date_string())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.description())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer, "{}", event.category())
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                writeln!(writer)
                    .map_err(|error| EventProviderError::OperationFailed(format!("{}", error)))?;
                Ok(())
            }
        };
    }
}
