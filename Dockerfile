FROM rust:1.76.0@sha256:a71cd88f9dd32fbdfa67c935f55165ddd89b7166e95de6c053c9bf33dd7381d5 as rust_builder

ARG TARGET=x86_64-unknown-linux-musl
ARG APPLICATION_NAME

RUN rustup target add ${TARGET}

RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/keep-cache

# borrowed (Ba Dum Tss!) from
# https://github.com/pablodeymo/rust-musl-builder/blob/7a7ea3e909b1ef00c177d9eeac32d8c9d7d6a08c/Dockerfile#L48-L49
RUN --mount=type=cache,target=/var/cache/apt --mount=type=cache,target=/var/lib/apt \
    apt-get update \
    && apt-get --no-install-recommends install -y \
        build-essential \
        musl \
        musl-dev \
        musl-tools \
    && rm -rf /var/lib/apt/lists

# The following block
# creates an empty app, and we copy in Cargo.toml and Cargo.lock as they represent our dependencies
# This allows us to copy in the source in a different layer which in turn allows us to leverage Docker's layer caching
# That means that if our dependencies don't change rebuilding is much faster
WORKDIR /build
RUN cargo new ${APPLICATION_NAME}
WORKDIR /build/${APPLICATION_NAME}
COPY Cargo.toml Cargo.lock ./

# because have our source in a subfolder, we need to ensure that the path in the [[bin]] section exists
RUN mkdir -p back-end/src && mv src/main.rs back-end/src/main.rs

RUN --mount=type=cache,id=cargo-dependencies,target=/build/${APPLICATION_NAME}/target \
    cargo build --release --target ${TARGET}

# TODO build JS

# now we copy in the rest
COPY . .

# --release not needed, it is implied with install
RUN --mount=type=cache,id=rust-full-build,target=/build/${APPLICATION_NAME}/target \
    cargo install --path . --target ${TARGET} --root /output

# ----
FROM node:21-alpine3.19@sha256:7bfef1d72befbb72b0894a3e4503edbdc0441058b4d091325143338cbf54cff8 as typescript_builder

# The following block
# creates an empty app, and we copy in package.json and packge-lock.json as they represent our dependencies
# This allows us to copy in the source in a different layer which in turn allows us to leverage Docker's layer caching
# That means that if our dependencies don't change rebuilding is much faster
WORKDIR /build
COPY package.json package-lock.json vite.config.ts tsconfig.json ./

RUN --mount=type=cache,id=npm-dependencies,target=/root/.npm \
    npm ci --include=dev

# now we copy in the rest
COPY front-end ./front-end/

RUN npm run build

FROM alpine:3.19.1@sha256:c5b1261d6d3e43071626931fc004f70149baeba2c8ec672bd4f27761f8e1ad6b

ARG APPLICATION_NAME

RUN addgroup -S appgroup && adduser -S appuser -G appgroup
USER appuser

WORKDIR /app

COPY --from=rust_builder /output/bin/* /app/entrypoint
COPY --from=typescript_builder /build/dist /app/dist

ENV RUST_BACKTRACE=full
ENTRYPOINT ["/app/entrypoint"]
