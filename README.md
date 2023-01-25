# L0C0B0T
Un B0T L0C0

## Requisitos

### Para correr el L0C0B0T

Si quieren correr el L0C0B0T en su máquina, necesitan tener instalado:

* [Python 3.8+](https://www.python.org/downloads/)
* [ffmpeg](https://ffmpeg.org/download.html) (creo, no estoy seguro)
* [yt-dlp](https://github.com/yt-dlp/yt-dlp)
* [Rust 1.66+](https://www.rust-lang.org/tools/install)

El L0C0B0T se compila y corre usando `cargo`:

```bash
cargo run
```

Además, necesitan tener un `.env` en el root del repositorio que defina la variable `DISCORD_TOKEN` con el token del bot. Por ejemplo:

```bash
# ./.env
DISCORD_TOKEN={{token}}
```

### Para correr el L0C0B0T en un container de Docker

La otra opción es correr el L0C0B0T en un container de [Docker](https://www.docker.com/). Se puede hacer de la siguiente manera:

```bash
docker build -t l0c0bot .
docker run -it --name l0c0bot l0c0bot
```

También pueden usar la imagen de Docker que subimos al GitHub Container Registry:

```bash
docker run -it ghcr.io/sp0nj4/l0c0b0t:latest
```
