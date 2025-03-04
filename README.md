# Rust Hour

A robust CRUD application built with Rust, featuring user authentication, question-answer functionality, and containerized deployment.

[![CI Pipeline](https://github.com/bjergsen243/rust_hour/actions/workflows/ci.yaml/badge.svg)](https://github.com/bjergsen243/rust_hour/actions/workflows/ci.yaml)

## Features

- **User Management**
  - Account creation and authentication
  - Email and password updates
  - JWT-based authentication
- **Question Management**
  - Create, read, update, and delete questions
  - Tag support for better categorization
  - Pagination support
- **Answer Management**
  - Create, read, update, and delete answers
  - Answer ownership tracking
  - Pagination support
- **API Documentation**
  - RESTful API endpoints
  - Comprehensive curl examples
- **Modern Development Setup**
  - Docker and Docker Compose support
  - GitHub Actions CI/CD pipeline
  - Code coverage and security scanning

## Prerequisites

Before you begin, ensure you have installed:

- [Rust](https://www.rust-lang.org/) (1.75 or later)
- [Docker](https://docs.docker.com/get-started/) (20.10 or later)
- [Docker Compose](https://docs.docker.com/compose/) (v2.0 or later)
- [PostgreSQL](https://www.postgresql.org/) (15 or later, if running locally)

## Quick Start

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

## Development Setup

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

## Test Coverage

Current test coverage is 24.79%. Our target is to maintain coverage above 80%. Here's how to check and improve test coverage:

1. Install cargo-tarpaulin:
   ```sh
   cargo install cargo-tarpaulin
   ```

2. Run coverage analysis:
   ```sh
   cargo tarpaulin
   ```

Areas needing coverage improvement:
- `src/store.rs` (0% coverage)
- `src/lib.rs` (0% coverage)
- `src/routes/authentication.rs` (27.76% coverage)
- `src/routes/question.rs` (52.08% coverage)

To improve coverage:
1. Add unit tests for untested functions
2. Include integration tests for API endpoints
3. Add property-based tests for complex logic
4. Ensure error cases are tested
5. Mock external dependencies where appropriate

## API Documentation

### Authentication

All authenticated endpoints require a JWT token in the Authorization header:

```sh
Authorization: Bearer <your-token>
```

### User Management

#### Create Account

```sh
curl -X POST 'localhost:8080/registration' \
  -H 'Content-Type: application/json' \
  -d '{"email": "example@gmail.com", "password": "123456789"}'
```

#### Login

```sh
curl -X POST 'localhost:8080/login' \
  -H 'Content-Type: application/json' \
  -d '{"email": "example@gmail.com", "password": "123456789"}'
```

#### Update Email

```sh
curl -X PUT 'localhost:8080/accounts' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"email": "update@gmail.com"}'
```

#### Update Password

```sh
curl -X PUT 'localhost:8080/accounts/update_password' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"password": "1234567890"}'
```

#### Get User Information

```sh
curl -X GET 'localhost:8080/accounts/me' \
  -H 'Authorization: Bearer <token>'
```

### Question Management

#### Create Question

```sh
curl -X POST 'localhost:8080/questions' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"title": "hello world", "content": "hello world", "tags": ["rust", "web"]}'
```

#### Update Question

```sh
curl -X PUT 'localhost:8080/questions/1' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"id": 1, "title": "update title", "content": "update content", "tags": ["updated"]}'
```

#### Delete Question

```sh
curl -X DELETE 'localhost:8080/questions/1' \
  -H 'Authorization: Bearer <token>'
```

#### List Questions

```sh
curl -X GET 'localhost:8080/questions?limit=20&offset=0'
```

#### Get Question Answers

```sh
curl -X GET 'localhost:8080/questions/1/answers?limit=20&offset=0'
```

### Answer Management

#### Create Answer

```sh
curl -X POST 'localhost:8080/answers' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"question_id": 2, "content": "answer question 2"}'
```

#### Update Answer

```sh
curl -X PUT 'localhost:8080/answers/1' \
  -H 'Authorization: Bearer <token>' \
  -H 'Content-Type: application/json' \
  -d '{"id": 1, "question_id": 2, "content": "update answer"}'
```

#### Delete Answer

```sh
curl -X DELETE 'localhost:8080/answers/1' \
  -H 'Authorization: Bearer <token>'
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

- Built with [actix-web](https://actix.rs/)
- Database powered by [PostgreSQL](https://www.postgresql.org/)
- Authentication using [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
