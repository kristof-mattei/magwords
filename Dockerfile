FROM --platform=${BUILDPLATFORM} rust:1.87.0@sha256:171cfe2c09e0d10b54aa52f5edcfab59e8073691f5dbc17128b11e48d2aad14a AS rust-base

ARG APPLICATION_NAME

RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache

# borrowed (Ba Dum Tss!) from
# https://github.com/pablodeymo/rust-musl-builder/blob/7a7ea3e909b1ef00c177d9eeac32d8c9d7d6a08c/Dockerfile#L48-L49
RUN --mount=type=cache,id=apt-cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-lib,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get --no-install-recommends install --yes \
        build-essential \
        musl-dev \
        musl-tools

FROM rust-base AS rust-linux-amd64
ARG TARGET=x86_64-unknown-linux-musl

FROM rust-base AS rust-linux-arm64
ARG TARGET=aarch64-unknown-linux-musl

FROM rust-${TARGETPLATFORM//\//-} AS rust-cargo-build

# expose (used in ./build.sh)
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

COPY ./setup-env.sh .
RUN --mount=type=cache,id=apt-cache,from=rust-base,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-lib,from=rust-base,target=/var/lib/apt,sharing=locked \
    ./setup-env.sh

RUN rustup target add ${TARGET}

# The following block
# creates an empty app, and we copy in Cargo.toml and Cargo.lock as they represent our dependencies
# This allows us to copy in the source in a different layer which in turn allows us to leverage Docker's layer caching
# That means that if our dependencies don't change rebuilding is much faster
WORKDIR /build
RUN cargo new ${APPLICATION_NAME}

WORKDIR /build/${APPLICATION_NAME}

COPY ./build.sh .

COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./

# because have our source in a subfolder, we need to ensure that the path in the [[bin]] section exists
RUN mkdir -p back-end/src && mv src/main.rs back-end/src/main.rs

RUN --mount=type=cache,target=/build/${APPLICATION_NAME}/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,id=cargo-registery,target=/usr/local/cargo/registry/,sharing=locked \
    ./build.sh build --release --target ${TARGET}

FROM rust-cargo-build AS rust-build

# expose (used in ./build.sh)
ARG BUILDPLATFORM
ARG TARGETPLATFORM
ARG TARGETARCH

WORKDIR /build/${APPLICATION_NAME}

# now we copy in the source which is more prone to changes and build it
COPY back-end ./back-end
COPY assets ./assets

# ensure cargo picks up on the change
RUN touch ./back-end/src/main.rs

# --release not needed, it is implied with install
RUN --mount=type=cache,target=/build/${APPLICATION_NAME}/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,id=cargo-registery,target=/usr/local/cargo/registry/,sharing=locked \
    ./build.sh install --path . --target ${TARGET} --root /output

FROM --platform=${BUILDPLATFORM} node:22.12.0-alpine3.19@sha256:40dc4b415c17b85bea9be05314b4a753f45a4e1716bb31c01182e6c53d51a654 AS typescript-build

# The following block
# creates an empty app, and we copy in package.json and packge-lock.json as they represent our dependencies
# This allows us to copy in the source in a different layer which in turn allows us to leverage Docker's layer caching
# That means that if our dependencies don't change rebuilding is much faster
WORKDIR /build
COPY package.json package-lock.json vite.config.ts tsconfig.json ./

ARG NPM_CONFIG_FUND=false
RUN --mount=type=cache,id=npm-dependencies,target=/root/.npm \
    npm i -g npm@latest \
    && npm ci --include=dev

# now we copy in the rest
COPY front-end ./front-end/

RUN npm run build

FROM --platform=${BUILDPLATFORM} alpine:3.21.3@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c AS passwd-build

# setting `--system` prevents prompting for a password
RUN addgroup --gid 900 appgroup \
    && adduser --ingroup appgroup --uid 900 --system --shell /bin/false appuser

RUN cat /etc/group | grep appuser > /tmp/group_appuser
RUN cat /etc/passwd | grep appuser > /tmp/passwd_appuser

FROM scratch

ARG APPLICATION_NAME

COPY --from=passwd-build /tmp/group_appuser /etc/group
COPY --from=passwd-build /tmp/passwd_appuser /etc/passwd

USER appuser

WORKDIR /app

COPY --from=rust-build /output/bin/${APPLICATION_NAME} /app/entrypoint
COPY --from=typescript-build /build/dist /app/dist

ENV RUST_BACKTRACE=full
ENTRYPOINT ["/app/entrypoint"]
