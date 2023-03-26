use chrono::{DateTime, NaiveDate, NaiveTime};
use icalendar::{Component, DatePerhapsTime, Event, EventLike};

use crate::notion::NotionDate;

#[derive(Default, Debug, Clone)]
pub struct DateEntry {
    pub id: String,
    pub title: String,
    pub date: Date,
    pub url: String,

    pub additional: Vec<(String, String)>,
}

impl DateEntry {
    pub fn as_event(&self) -> Event {
        // Create calendar event
        let mut event = Event::new();
        event.uid(&self.id);

        // Summary
        event.summary(&self.title);

        // Description text
        let mut description = String::new();
        for (name, value) in &self.additional {
            description += &format!("{}: {}\n", name, value);
        }
        description += &format!("---\n{}", self.url);
        event.description(&description);

        // Starts
        if let Some(start_time) = self.date.start_time {
            // Event with date + time
            event.starts(DatePerhapsTime::from(
                self.date.start_date.and_time(start_time),
            ));
        } else {
            // Event without time
            event.starts(DatePerhapsTime::from(self.date.start_date));
        }

        // Ends
        if let Some(end_date) = self.date.end_date {
            if let Some(end_time) = self.date.end_time {
                // Event with date + time
                event.ends(DatePerhapsTime::from(end_date.and_time(end_time)));
            } else {
                // Event without time
                event.ends(DatePerhapsTime::from(end_date));
            }
        } else {
            // If the event has a start time, it also needs a end time.
            // If it isn't set on notion side, we just set a duration of one hour.
            if let Some(start_time) = self.date.start_time {
                let starts = self.date.start_date.and_time(start_time);
                let ends = starts + chrono::Duration::hours(1);
                event.ends(DatePerhapsTime::from(ends));
            }
        }

        event.done()
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
