# grammys-voting-service/Dockerfile (Producción)

# Etapa 1: Compilar el binario usando un tag ligero de la lista
FROM rust:1-slim-bookworm AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Etapa 2: Imagen de ejecución final limpia basada en el mismo OS
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/voting-service /app/voting-service
EXPOSE 8080
CMD ["./voting-service"]
