use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Local, NaiveDate};
use sqlite::{Connection, State};

use crate::EventProvider;
use crate::events::{Category, Event, MonthDay, Rule};
use crate::filters::EventFilter;
use crate::providers::EventProviderError;

#[allow(dead_code)]
fn make_date_part(filter: &EventFilter) -> String {
    if let Some(month_day) = filter.month_day() {
        let md = format!("{:02}-{:02}", month_day.month(), month_day.day());
        format!("strftime('%m-%d', event_date) = '{}'", md)
    } else {
        "".to_string()
    }
}
fn make_category_part(filter: &EventFilter, category_map: &HashMap<i64, Category>) -> String {
    if let Some(filter_category) = filter.category() {
        let mut filter_category_id: Option<i64> = None;

        // Brute force search for maching category:
        for (category_id, category) in category_map {
            if *category == filter_category {
                filter_category_id = Some(*category_id);
                break;
            }
        }

        match filter_category_id {
            Some(id) => format!("category_id = {}", id),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    }
}

fn make_text_part(filter: &EventFilter) -> String {
    if let Some(text) = filter.text() {
        format!("event_description LIKE '%{}%'", text)
    } else {
        "".to_string()
    }
}

fn make_where_clause(
    filter: &EventFilter,
    category_map: &HashMap<i64, Category>,
) -> Result<String, EventProviderError> {
    let mut parts: Vec<String> = Vec::new();

    /*if filter.contains_month_day() {
        parts.push(make_date_part(filter));
    }*/

    if filter.contains_category() {
        let category_part = make_category_part(filter, category_map);
        if !category_part.is_empty() {
            parts.push(category_part);
        } else {
            return Ok("WHERE 0".to_string());
        }
    }

    if filter.contains_text() {
        parts.push(make_text_part(filter));
    }

    let mut result = String::new();
    if !parts.is_empty() {
        result.push_str("WHERE ");
        result.push_str(&parts.join(" AND "));
    }

    Ok(result)
}

fn category_exists(
    category_map: &HashMap<i64, Category>,
    category: &Category,
    id: &mut i64,
) -> bool {
    for (_category_id, category_iterator) in category_map {
        if category_iterator == category {
            *id = *_category_id;
            return true;
        }
    }
    false
}

pub struct SQLiteProvider {
    name: String,
    path: PathBuf,
}

impl SQLiteProvider {
    pub fn new(name: &str, path: &Path) -> Self {
        Self {
            name: name.to_string(),
            path: path.to_path_buf(),
        }
    }

    fn get_categories(
        &self,
        connection: &Connection,
    ) -> Result<HashMap<i64, Category>, EventProviderError> {
        let mut category_map: HashMap<i64, Category> = HashMap::new();
        let category_query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = connection
            .prepare(category_query)
            .map_err(|error| EventProviderError::Db(format!("{}", error)))?;

        while let Ok(State::Row) = statement.next() {
            let category_id = statement
                .read::<i64, _>("category_id")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            let primary = statement
                .read::<String, _>("primary_name")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            let secondary = statement
                .read::<Option<String>, _>("secondary_name")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;

            let category = match secondary {
                Some(sec) => Category::new(&primary, &sec),
                None => Category::from_primary(&primary),
            };
            category_map.insert(category_id, category);
        }

        Ok(category_map)
    }
}

impl EventProvider for SQLiteProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(
        &self,
        filter: &EventFilter,
        events: &mut Vec<Event>,
    ) -> Result<(), EventProviderError> {
        let connection = Connection::open(self.path.clone())
            .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
        let category_map = self.get_categories(&connection)?;

        let where_clause = make_where_clause(filter, &category_map)?;
        let mut event_query: String =
            "SELECT event_date, event_description, category_id FROM event".to_string();
        if !where_clause.is_empty() {
            event_query.push(' ');
            event_query.push_str(&where_clause);
        }

        let mut statement = connection
            .prepare(event_query)
            .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
        while let Ok(State::Row) = statement.next() {
            let mut date_string = statement
                .read::<String, _>("event_date")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            let description = statement
                .read::<String, _>("event_description")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            let category_id = statement
                .read::<i64, _>("category_id")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            let category = category_map.get(&category_id).ok_or_else(|| {
                EventProviderError::Db(format!("missing category id {}", category_id))
            })?;

            let is_yearless = date_string.starts_with("--");
            if is_yearless {
                let today: NaiveDate = Local::now().date_naive();
                let year_string = format!("{:04}-", today.year());
                date_string = date_string.replace("--", &year_string);
            }

            let event: Event;

            if let Ok(date) = NaiveDate::parse_from_str(&date_string, "%F") {
                if is_yearless {
                    event = Event::new_annual(
                        MonthDay::new(date.month(), date.day()),
                        description.clone(),
                        category.clone(),
                    );
                } else {
                    event = Event::new_singular(date, description.clone(), category.clone());
                }
                if filter.accepts(&event) {
                    events.push(event);
                }
            } else if let Some(rule) = Rule::parse(&date_string) {
                event = Event::new_rule_based(rule, description.clone(), category.clone());
                if filter.accepts(&event) {
                    events.push(event);
                }
            } else {
                eprintln!("Invalid date in SQLite row: {}", date_string);
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
        let connection = Connection::open(self.path.clone())
            .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
        let category_map = self.get_categories(&connection)?;
        let category = _event.category();

        let mut category_id: i64 = 0;
        if !category_exists(&category_map, &category, &mut category_id) {
            let primary = category.primary();
            let secondary = category.secondary().to_string();

            let query = if secondary.is_empty() {
                format!(
                    "INSERT INTO category (primary_name, secondary_name)
                    VALUES ('{}', NULL)
                    RETURNING category_id",
                    primary
                )
            } else {
                format!(
                    "INSERT INTO category (primary_name, secondary_name)
                    VALUES ('{}', '{}')
                    RETURNING category_id",
                    primary, secondary
                )
            };

            let mut statement = connection
                .prepare(&query)
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            statement
                .next()
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
            category_id = statement
                .read::<i64, _>("category_id")
                .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
        }

        let date = _event.date_string();
        let description = _event.description();
        let query = format!(
            "INSERT INTO event (event_date, event_description, category_id)
            VALUES ('{}', '{}', {})",
            date, description, category_id
        );

        connection
            .execute(query)
            .map_err(|error| EventProviderError::Db(format!("{}", error)))?;
        Ok(())
    }
}
