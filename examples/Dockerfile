FROM rust:1.77.0
WORKDIR /app
COPY . .
RUN  ["cargo", "build", "--bin", "docker"] 
CMD ["cargo", "run", "--bin", "docker"] 
