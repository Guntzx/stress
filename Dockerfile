# Dockerfile para compilar test-stress
FROM rust:1.75-slim as builder

# Instalar dependencias necesarias
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    && rm -rf /var/lib/apt/lists/*

# Agregar targets para múltiples plataformas
RUN rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-msvc

# Crear directorio de trabajo
WORKDIR /app

# Copiar archivos del proyecto
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Compilar para Linux
RUN cargo build --release --target x86_64-unknown-linux-gnu

# Compilar para Windows (si es posible)
RUN cargo build --release --target x86_64-pc-windows-msvc || echo "Windows build failed - requires Windows SDK"

# Etapa final para crear imagen ligera
FROM debian:bullseye-slim

# Instalar dependencias de runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copiar ejecutable
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/test-stress /usr/local/bin/

# Crear directorios necesarios
RUN mkdir -p /app/configs /app/results /app/logs

WORKDIR /app

# Exponer puerto si es necesario
EXPOSE 8080

# Comando por defecto
CMD ["test-stress", "--help"] 