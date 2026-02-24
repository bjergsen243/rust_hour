-- Initialize separate databases for each microservice
-- This script runs once when the postgres container is first created

CREATE DATABASE auth_db;
CREATE DATABASE monitor_db;
CREATE DATABASE alert_db;
CREATE DATABASE saga_db;
