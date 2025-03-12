# Rust Hour

A robust CRUD application built with Rust, featuring user authentication, question-answer functionality, and containerized deployment.

[![CI Pipeline](https://github.com/bjergsen243/rust_hour/actions/workflows/ci.yaml/badge.svg)](https://github.com/bjergsen243/rust_hour/actions/workflows/ci.yaml)

## Installation and Usage

### Prerequisites

Before you begin, ensure you have installed:

- [Rust](https://www.rust-lang.org/) (1.75 or later)
- [Docker](https://docs.docker.com/get-started/) (20.10 or later)
- [Docker Compose](https://docs.docker.com/compose/) (v2.0 or later)
- [PostgreSQL](https://www.postgresql.org/) (15 or later, if running locally)

### Quick Start

1. Clone the repository:

   ```sh
   git clone https://github.com/bjergsen243/rust_hour
   cd rust_hour
   ```

2. Copy the environment file and configure it:

   ```sh
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. Start the application using Docker:

   ```sh
   docker compose up -d
   ```

   The API will be available at `http://localhost:8080`

### Development Setup

1. Install Rust dependencies:

   ```sh
   cargo build
   ```

2. Run the tests:

   ```sh
   cargo test
   ```

3. Start the development server:

   ```sh
   cargo run
   ```

## API Endpoints

| Endpoint                        | Description                                       |
| ------------------------------- | ------------------------------------------------- |
| `POST /registration`            | Create a new user account                         |
| `POST /login`                   | Authenticate a user and obtain a JWT token        |
| `PUT /accounts`                 | Update user email                                 |
| `PUT /accounts/update_password` | Update user password                              |
| `GET /accounts/me`              | Retrieve information about the authenticated user |
| `POST /questions`               | Create a new question                             |
| `PUT /questions/{id}`           | Update an existing question                       |
| `DELETE /questions/{id}`        | Delete a question                                 |
| `GET /questions`                | List questions with optional pagination           |
| `GET /questions/{id}/answers`   | Get answers for a specific question               |
| `POST /answers`                 | Create a new answer                               |
| `PUT /answers/{id}`             | Update an existing answer                         |
| `DELETE /answers/{id}`          | Delete an answer                                  |
