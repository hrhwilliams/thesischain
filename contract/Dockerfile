FROM ghcr.io/foundry-rs/foundry:latest
WORKDIR /app
COPY . .
RUN forge build
USER root
RUN mkdir -p /shared && chmod 777 /shared
USER foundry
ENTRYPOINT ["/app/deploy.sh"]
