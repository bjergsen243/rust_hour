#!/bin/bash

# Load environment variables from .env file
if [ -f ../.env ]; then
    echo "Loading environment variables from .env file"
    export $(grep -v '^#' ../.env | xargs)
else
    echo "Error: .env file not found in parent directory"
    exit 1
fi

# Function to run SQL files using Docker
run_sql_file() {
    echo "Running $1..."
    docker exec -i postgres_db psql -U $DB_USER -d $DB_NAME -f - < "$1"
    if [ $? -ne 0 ]; then
        echo "Error running $1"
        exit 1
    fi
    echo "Successfully ran $1"
}

# Create database if it doesn't exist
echo "Creating database $DB_NAME if it doesn't exist..."
docker exec -i postgres_db psql -U $DB_USER -c "CREATE DATABASE $DB_NAME;" 2>/dev/null
echo "Database $DB_NAME created or already exists"

# Run migrations in order
echo "Running migrations..."
run_sql_file "20240221183024_questions_table.up.sql"
run_sql_file "20240221183051_answers_table.up.sql"
run_sql_file "20240221183350_accounts_tables.up.sql"

echo "All migrations completed successfully!" 