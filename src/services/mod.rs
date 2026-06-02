use crate::models::VoteCreatedEvent;
use crate::models::{CreateVoteDto, Vote};
use chrono::Utc;
use futures_util::stream::TryStreamExt;
use lapin::{BasicProperties, Channel, options::BasicPublishOptions};
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

    pub async fn publish_vote_event(channel: &Channel, vote: &Vote) -> Result<(), lapin::Error> {
        // 1. Construir el contrato del evento ligero
        let event = VoteCreatedEvent {
            vote_id: vote.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            user_id: vote.user_id.clone(),
            category_id: vote.category_id.clone(),
            nominee_id: vote.nominee_id.clone(),
            timestamp: vote.created_at.to_rfc3339(), // Formato de fecha estándar para el ecosistema
        };

        // 2. Serializar el struct de Rust a un Vector de Bytes (JSON)
        let payload = serde_json::to_vec(&event)
            .expect("Error fatal: No se pudo serializar el evento de votación");

        // 3. Disparar el mensaje de forma asíncrona a RabbitMQ
        channel
            .basic_publish(
                "votes.exchange".into(), // Nombre del Exchange (lo declararemos en el broker)
                "vote.created".into(),   // Routing Key (Criterio de enrutamiento)
                BasicPublishOptions::default(),
                &payload,
                BasicProperties::default().with_content_type("application/json".into()),
            )
            .await?;

        tracing::info!(
            "[RABBITMQ] Evento 'vote.created' publicado con éxito para el voto: {}",
            event.vote_id
        );
        Ok(())
    }
}
