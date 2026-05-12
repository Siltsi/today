use reqwest::{blocking::Client, blocking::Response};
use serde::Deserialize;
use chrono::NaiveDate;

use crate::EventProvider;
use crate::providers::EventProviderError;
use crate::events::{Category, Event};
use crate::filters::EventFilter;

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
    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        // We need a date parameter for the URL, so if the filter
        // does not specify it, we are done.
        if filter.month_day().is_none() {
            eprintln!("No month-day in filter");
            return;
        }
        let month_day = filter.month_day().unwrap();
        let date_parameter = format!("date={:02}-{:02}", month_day.month(), month_day.day());
        let url = format!("{}?{}", &self.url, date_parameter);
        let client = Client::new();
        let request = client.get(&url).send();
        let response: Response;
        if request.is_err() {
            eprintln!("Error while retrieving data: {:#?}", request.err());
            return;
        } else {
            response = request.ok().unwrap();
        }
        let json_events = response.json::<Vec<JSONEvent>>().unwrap();
        for json_event in json_events {
            let date = NaiveDate::parse_from_str(&json_event.date, "%F").unwrap();
            let category = Category::from_str(&json_event.category);
            let event = Event::new_singular(date, json_event.description, category);
            if filter.accepts(&event) {
                events.push(event);
            }
        }
    }

    fn is_add_supported(&self) -> bool {
        false
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        Err(EventProviderError::OperationNotSupported)
    }
}
