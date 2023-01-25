### Stage 1: Build ###
FROM rust:1.66.1 as build

WORKDIR /usr/src/L0C0B0T

# Install aditional build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends cmake

# Add source code
COPY ["Cargo.toml", "Cargo.lock", "./"] 
ADD src/ src/

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake

# Compile and install
RUN cargo install --path .

### Stage 2: Run ###
FROM debian:stable-slim

ARG EXT_DISCORD_TOKEN
ENV DISCORD_TOKEN=$EXT_DISCORD_TOKEN

# Install aditional runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ffmpeg \
    python3 \
    python3-pip

# Install yt-dlp
RUN pip3 install --no-cache --upgrade yt-dlp

# Copy the binary from the build stage
COPY --from=build /usr/local/cargo/bin/l0c0b0t /usr/local/bin/L0C0B0T

# Run the binary
ENTRYPOINT ["L0C0B0T"]
