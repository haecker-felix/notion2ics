use chrono::{DateTime, NaiveDate, NaiveTime};
use icalendar::{Component, DatePerhapsTime, Event, EventLike};

use crate::notion::NotionDate;

#[derive(Default, Debug, Clone)]
pub struct DateEntry {
    pub id: String,
    pub emoji: Option<String>,
    pub title: String,
    pub date: Date,
    pub url: String,

    pub additional: Vec<(String, String)>,
}

impl DateEntry {
    pub fn as_event(&self) -> Event {
        // Summary
        let summary = if let Some(emoji) = &self.emoji {
            format!("{} {}", emoji, self.title)
        } else {
            self.title.clone()
        };

        // Description text
        let mut description = String::new();
        for (name, value) in &self.additional {
            description += &format!("{}: {}\n", name, value);
        }
        description += &format!("---\n{}", self.url);

        // Starts
        let starts: DatePerhapsTime = if let Some(start_time) = self.date.start_time {
            // Event with date + time
            DatePerhapsTime::DateTime(self.date.start_date.and_time(start_time).into())
        } else {
            // Event without time
            DatePerhapsTime::Date(self.date.start_date)
        };

        let ends = if let Some(end_date) = self.date.end_date {
            if let Some(end_time) = self.date.end_time {
                // Event with date + time
                DatePerhapsTime::DateTime(end_date.and_time(end_time).into())
            } else {
                // Event without time
                DatePerhapsTime::Date(end_date)
            }
        } else {
            // no end date set
            starts.clone()
        };

        // Create calendar event
        Event::new()
            .starts(starts)
            .ends(ends)
            .summary(&summary)
            .description(&description)
            .done()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Date {
    start_date: NaiveDate,
    start_time: Option<NaiveTime>,
    end_date: Option<NaiveDate>,
    end_time: Option<NaiveTime>,
}

impl TryFrom<NotionDate> for Date {
    type Error = chrono::ParseError;

    fn try_from(value: NotionDate) -> Result<Self, Self::Error> {
        let mut date = Self::default();

        // start
        if let Ok(start) = DateTime::parse_from_rfc3339(&value.start) {
            date.start_date = start.date_naive();
            date.start_time = Some(start.time());
        } else {
            date.start_date = NaiveDate::parse_from_str(&value.start, "%Y-%m-%d")?;
            date.start_time = None;
        }

        // end (optional)
        if let Some(end) = &value.end {
            if let Ok(end) = DateTime::parse_from_rfc3339(&end) {
                date.end_date = Some(end.date_naive());
                date.end_time = Some(end.time());
            } else {
                date.end_date = Some(NaiveDate::parse_from_str(&value.start, "%Y-%m-%d")?);
                date.end_time = None;
            }
        }

        Ok(date)
    }
}
