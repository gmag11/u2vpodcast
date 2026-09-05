###############################################################################
## Backend builder
###############################################################################
FROM rust:alpine3.24 AS backend_builder

LABEL maintainer="Lorenzo Carbonell <a.k.a. atareao> lorenzo.carbonell.cerezo@gmail.com"

RUN apk add --update --no-cache \
            gcc \
            musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src src

RUN cargo build --release && \
    cp /app/target/release/u2vpodcast /app/u2vpodcast

###############################################################################
## Frontend builder
###############################################################################
FROM node:24-alpine AS frontend_base
ENV PNPM_HOME=/pnpm
ENV PATH="${PNPM_HOME}:${PATH}"
RUN corepack enable
COPY ./frontend/ /app
WORKDIR /app

FROM frontend_base AS frontend_builder
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install \
    --frozen-lockfile && \
    pnpm test && \
    pnpm run build

###############################################################################
## Final image
###############################################################################
FROM alpine:3.24

ENV USER=app
ENV UID=10001

RUN apk add --update --no-cache \
            deno \
            ffmpeg~=8.1 \
            python3~=3.14 && \
    rm -rf /var/cache/apk && \
    rm -rf /var/lib/app/lists*

# Copy from backend_builder
COPY --from=backend_builder /app/u2vpodcast /app/
COPY --from=frontend_builder /app/dist /app/html

COPY migrations/ /app/migrations/

# Create the user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/${USER}" \
    --shell "/sbin/nologin" \
    --uid "${UID}" \
    "${USER}" && \
    mkdir -p /app/db /app/audios && \
    chown -R app: /app

# Set the work dir
WORKDIR /app
USER app

# Install the official standalone yt-dlp binary from GitHub releases. It is a
# zipapp that runs on the system python3 (kept above). Using the standalone
# binary lets `yt-dlp --update` self-update from the official
# github.com/yt-dlp/yt-dlp releases channel, keeping "always latest" without a
# PyPI pip dependency. `python3` and `deno` are the runtime requirements
# (deno/node as the JS runtime for signature solving).
RUN mkdir -p /app/.local/bin && \
    wget -qO /app/.local/bin/yt-dlp \
        https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp && \
    chmod +x /app/.local/bin/yt-dlp

CMD ["/app/u2vpodcast"]
