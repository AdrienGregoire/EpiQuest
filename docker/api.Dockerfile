FROM rust:1

WORKDIR /app
CMD ["cargo", "run", "-p", "api"]
