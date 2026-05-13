use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;
use sqlite::{Connection, State};

use crate::EventProvider;
use crate::events::{Category, Event};
use crate::filters::EventFilter;
use crate::providers::EventProviderError;

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
) -> Result<String, ()> {
    let mut parts: Vec<String> = Vec::new();

    if filter.contains_month_day() {
        parts.push(make_date_part(filter));
    }

    if filter.contains_category() {
        let category_part = make_category_part(filter, category_map);
        if category_part != "" {
            parts.push(category_part);
        } else {
            return Err(());
        }
    }

    if filter.contains_text() {
        parts.push(make_text_part(filter));
    }

    let mut result = "".to_string();
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

    fn get_categories(&self, connection: &Connection) -> HashMap<i64, Category> {
        let mut category_map: HashMap<i64, Category> = HashMap::new();
        let category_query = "SELECT category_id, primary_name, secondary_name FROM category";
        let mut statement = connection.prepare(category_query).unwrap();
        while let Ok(State::Row) = statement.next() {
            let category_id = statement.read::<i64, _>("category_id").unwrap();
            let primary = statement.read::<String, _>("primary_name").unwrap();
            let secondary = statement
                .read::<Option<String>, _>("secondary_name")
                .unwrap();

            let category = match secondary {
                Some(sec) => Category::new(&primary, &sec),
                None => Category::from_primary(&primary),
            };
            category_map.insert(category_id, category);
        }

        category_map
    }
}

impl EventProvider for SQLiteProvider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn get_events(&self, filter: &EventFilter, events: &mut Vec<Event>) {
        let connection = Connection::open(self.path.clone()).unwrap();
        let category_map = self.get_categories(&connection);

        let where_clause = match make_where_clause(filter, &category_map) {
            Ok(wc) => wc,
            Err(_) => {
                return;
            }
        };
        let mut event_query: String =
            "SELECT event_date, event_description, category_id FROM event".to_string();
        event_query.push(' '); // space between table name and WHERE clause
        event_query.push_str(&where_clause);

        let mut statement = connection.prepare(event_query).unwrap();
        while let Ok(State::Row) = statement.next() {
            let date_string = statement.read::<String, _>("event_date").unwrap();
            let date = NaiveDate::parse_from_str(&date_string, "%F").unwrap();
            let description = statement.read::<String, _>("event_description").unwrap();
            let category_id = statement.read::<i64, _>("category_id").unwrap();
            let category = category_map.get(&category_id).unwrap();
            events.push(Event::new_singular(
                date,
                description.to_string(),
                category.clone(),
            ));
        }
    }

    fn is_add_supported(&self) -> bool {
        true
    }

    fn add_event(&self, _event: &Event) -> Result<(), EventProviderError> {
        if !self.is_add_supported() {
            return Err(EventProviderError::OperationNotSupported);
        }
        let connection = Connection::open(self.path.clone()).unwrap();
        let category_map = self.get_categories(&connection);
        let category = _event.category();

        let mut category_id: i64 = 0;
        if !category_exists(&category_map, &category, &mut category_id) {
            let primary = category.primary();
            let mut secondary = category.secondary().to_string();

            if secondary.is_empty() {
                secondary = "NULL".to_string();
            }

            let query = format!(
                "INSERT INTO category (primary_name, secondary_name)
                VALUES ('{}', '{}')
                RETURNING category_id",
                primary, secondary
            );

            let mut statement = connection.prepare(&query).unwrap();
            statement.next().unwrap();
            category_id = statement.read::<i64, _>("category_id").unwrap();
        }

        let date = _event.date_string();
        let description = _event.description();
        let query = format!(
            "INSERT INTO event (event_date, event_description, category_id)
            VALUES ('{}', '{}', {})",
            date, description, category_id
        );

        connection.execute(query).unwrap();
        Ok(())
    }
}
