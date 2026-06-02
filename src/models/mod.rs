use chrono::{DateTime, Utc};
pub use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

fn serialize_option_object_id_as_hex_string<S>(
    val: &Option<ObjectId>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match val {
        Some(oid) => {
            mongodb::bson::serde_helpers::serialize_object_id_as_hex_string(oid, serializer)
        }
        None => serializer.serialize_none(),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    #[serde(
        rename = "_id",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_object_id_as_hex_string"
    )]
    pub id: Option<ObjectId>,

    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "categoryId")]
    pub category_id: String,

    #[serde(rename = "nomineeId")]
    pub nominee_id: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoteDto {
    #[serde(rename = "userId")]
    pub user_id: String,

    #[serde(rename = "categoryId")]
    pub category_id: String,

    #[serde(rename = "nomineeId")]
    pub nominee_id: String,
}
