use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionDatabaseResponse {
    pub object: String,
    #[serde(rename = "results")]
    pub pages: Vec<NotionPage>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionPage {
    pub id: String,
    pub icon: Option<NotionIcon>,
    pub url: String,
    pub properties: HashMap<String, NotionProperty>,
    pub created_time: String,
    pub last_edited_time: String,
    pub archived: bool,
}

impl NotionPage {
    pub fn title(&self) -> String {
        let mut title = String::new();

        for property in self.properties.values() {
            match &property.value {
                NotionPropertyValue::Title(value) => {
                    // Assume that the first array entry is the text
                    if let Some(value) = value.first() {
                        title = value.plain_text.clone();
                    }
                }
                _ => (),
            }
        }

        // Optional page emoji
        if let Some(emoji) = &self.icon.clone().map(|i| i.emoji).flatten() {
            title = format!("{emoji} {title}");
        }

        title
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionIcon {
    pub emoji: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionProperty {
    #[serde(flatten)]
    pub value: NotionPropertyValue,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotionPropertyValue {
    /* Properties we care about */
    Date(Option<NotionDate>),
    Title(Vec<NotionText>),
    Number(Option<f64>),
    RichText(Vec<NotionText>),
    Url(Option<String>),
    Select(Option<NotionSelect>),
    MultiSelect(Vec<NotionSelect>),
    Relation(Vec<NotionRelation>),
    People(Vec<NotionPerson>),
    Status(NotionSelect),

    /* Ignored properties */
    Checkbox(Value),
    CreatedBy(Value),
    CreatedTime(Value),
    Email(Value),
    Files(Value),
    Formula(Value),
    LastEditedBy(Value),
    LastEditedTime(Value),
    PhoneNumber(Value),
    Rollup(Value),

    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionText {
    #[serde(rename = "type")]
    pub type_: NotionTextType,
    pub plain_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotionTextType {
    Mention,
    #[default]
    Text,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionDate {
    pub start: String,
    pub end: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionRelation {
    pub id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionPerson {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotionSelectType {
    // Standard notion values
    NotStarted,
    InProgress,
    Done,
    // User-created status
    #[serde(other)]
    #[default]
    Unknown,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotionSelect {
    pub id: NotionSelectType,
    pub name: String,
}
