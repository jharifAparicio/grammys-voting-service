use std::env;

pub struct AppConfig {
    pub port: u16,
    pub mongo_uri: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("La variable PORT debe ser un número entero válido");

        // Lee la URI de MongoDB inyectada por Docker, o usa localhost por si haces pruebas nativas
        let mongo_uri = env::var("MONGO_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017/votes_db".to_string());

        AppConfig { port, mongo_uri }
    }
}
