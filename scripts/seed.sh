#!/bin/bash
cat scripts/seed_trivia.sql | docker compose exec -T db psql -U user -d daily_challenge
cat scripts/seed_code_output.sql | docker compose exec -T db psql -U user -d daily_challenge
echo "Database seeded successfully."