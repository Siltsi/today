use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use csv::{ReaderBuilder, WriterBuilder};

use crate::EventProvider;
use crate::events::{Category, Event};
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

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let mut reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(self.path.clone())
            .expect("existing CSV file");

        for result in reader.records() {
            let record = result.unwrap();

            let date_string = record[0].to_string();
            let description = record[1].to_string();
            let category_string = record[2].to_string();

            match NaiveDate::parse_from_str(&date_string, "%F") {
                Ok(date) => {
                    let category = Category::from_str(&category_string);
                    let event = Event::new_singular(date, description.clone(), category);
                    if filter.accepts(&event) {
                        events.push(event);
                    }
                }
                Err(_) => {
                    eprintln!("Invalid date '{}'", date_string);
                }
            }
        }
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        if !self.is_add_supported() {
            return Err(EventProviderError::OperationNotSupported);
        }
        
        let mut writer = WriterBuilder::new()
            .has_headers(false)
            .from_path(self.path.clone())
            .map_err(|_| EventProviderError::OperationFailed)?;

        let date = _event.date_string();
        let description = _event.description();
        let category = _event.category().to_string();

        writer.write_record(&[date, description, category]).map_err(|_| EventProviderError::OperationFailed)?;
        writer.flush().map_err(|_| EventProviderError::OperationFailed)?;
        Ok(())
    }
}
