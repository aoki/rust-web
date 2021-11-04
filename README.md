```console
cargo install diesel_cli --no-default-features --features postgres
docker-compose up -d
export DATABASE_URL=postgresql://postgres:password@localhost:5432/log-collector
cd server && cp .local.env .env
```

```
diesel setup          # init
diesel migration run  # up
diesel migration redo # down
```
