docker compose -f docker-compose.prod.yml down
docker compose -f docker-compose.prod.yml up bigchaindb -d
docker logs -f sogolscdb-bigchaindb-1