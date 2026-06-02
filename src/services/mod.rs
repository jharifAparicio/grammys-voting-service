use crate::models::{CreateVoteDto, Vote};
use chrono::Utc;
use futures_util::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::error::Error;
use mongodb::{Collection, Database};

pub struct VoteService;

impl VoteService {
    // Registrar un voto
    pub async fn create_vote(db: &Database, dto: CreateVoteDto) -> Result<Vote, Error> {
        let collection: Collection<Vote> = db.collection("votes");

        let new_vote = Vote {
            id: None, // Dejamos que Mongo maneje el ObjectId nativo
            user_id: dto.user_id,
            category_id: dto.category_id,
            nominee_id: dto.nominee_id,
            created_at: Utc::now(),
        };

        // Inserción asíncrona
        let insert_result = collection.insert_one(new_vote.clone(), None).await?;

        let mut saved_vote = new_vote;
        if let Some(inserted_id) = insert_result.inserted_id.as_object_id() {
            saved_vote.id = Some(inserted_id);
        }

        Ok(saved_vote)
    }

    // Obtener todos los votos
    pub async fn find_all(db: &Database) -> Result<Vec<Vote>, Error> {
        let collection: Collection<Vote> = db.collection("votes");
        let cursor = collection.find(doc! {}, None).await?;

        // Gracias a TryStreamExt, podemos usar try_collect
        let votes: Vec<Vote> = cursor.try_collect().await?;
        Ok(votes)
    }

    // Obtener votos por usuario
    pub async fn find_by_user(db: &Database, user_id: &str) -> Result<Vec<Vote>, Error> {
        let collection: Collection<Vote> = db.collection("votes");
        let filter = doc! { "userId": user_id };
        let cursor = collection.find(filter, None).await?;

        let votes: Vec<Vote> = cursor.try_collect().await?;
        Ok(votes)
    }
}
