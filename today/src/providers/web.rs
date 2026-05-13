use chrono::NaiveDate;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::EventProvider;
use crate::events::{Category, Event};
use crate::filters::EventFilter;
use crate::providers::EventProviderError;

#[derive(Deserialize, Debug)]
struct JSONEvent {
    category: String,
    date: String,
    description: String,
}

pub struct WebProvider {
    name: String,
    url: String,
}

impl WebProvider {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

impl EventProvider for WebProvider {
    fn name(&self) -> String {
        self.name.clone()
    }
    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) -> Result<(), EventProviderError> {
        let month_day = match filter.month_day() {
            Some(month_day) => month_day,
            None => return Err(EventProviderError::Parse("missing month-day filter".to_string())),
        };
        let date_parameter = format!("date={:02}-{:02}", month_day.month(), month_day.day());
        let url = format!("{}?{}", &self.url, date_parameter);
        let client = Client::new();
        let response = client
            .get(&url)
            .send()
            .map_err(|error| EventProviderError::Http(format!("{}", error)))?;

        let json_events = response
            .json::<Vec<JSONEvent>>()
            .map_err(|error| EventProviderError::Http(format!("{}", error)))?;
        for json_event in json_events {
            let date = match NaiveDate::parse_from_str(&json_event.date, "%F") {
                Ok(date) => date,
                Err(error) => {
                    eprintln!("Invalid date in web payload: {}", error);
                    continue;
                }
            };
            let category = Category::from_str(&json_event.category);
            let event = Event::new_singular(date, json_event.description, category);
            if filter.accepts(&event) {
                events.push(event);
            }
        }

        Ok(())
    }

    fn is_add_supported(&self) -> bool {
        false
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        if !self.is_add_supported() {
            return Err(EventProviderError::OperationNotSupported);
        }
        Ok(())
    }
}
