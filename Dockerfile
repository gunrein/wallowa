# syntax=docker/dockerfile:1

FROM rust:latest AS base
RUN apt-get update
RUN apt-get install -y ca-certificates curl gnupg
RUN mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg
ENV NODE_MAJOR=20
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN apt-get update
RUN apt-get install -y nodejs libssl-dev git g++ cmake ninja-build
WORKDIR /usr/wallowa
RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    npm ci
COPY ./ ./
RUN npm run clean
RUN npm run build:css
RUN npm run build:esbuild
RUN npm run build:static
RUN npm run build:backend

FROM ubuntu:23.10
RUN apt-get update
RUN apt-get install -y ca-certificates
RUN mkdir /usr/wallowa
WORKDIR /usr/wallowa
COPY --from=base /usr/wallowa/target/release/wallowa /usr/local/bin/wallowa
ENTRYPOINT ["wallowa"]
EXPOSE 9843
