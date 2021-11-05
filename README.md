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

```
curl -v -H 'Content-Type: application/json' -d'{"user_agent": "Mozilla", "response_time": 200}' localhost:3000/logs
docker-compose exec postgresql psql -U postgres log_collector '-c SELECT * FROM logs;'
curl localhost:3000/logs
curl 'localhost:3000/logs?from=2021-11-04T10:50:52.082476Z&until=2021-11-04T12:57:12.431492Z'
```
