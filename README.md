# Rust Hour
## _The Simple CRUD Rust Application_

Rust Hour allows users to create questions and answer them via HTTP.

## Features

- Users can create accounts and log in.
- Users can update email, password.
- Users can create questions with a title, content, and tags.
- Users can update questions.
- Users can delete questions.
- Users can answers questions with a content.
- User can update answers.
- Users can delete answers.

## Prerequisites

Make sure you installed those things before started!

- [Rust] - Rust for programming.
- [Docker & Docker Compose] - Docker and Docker Compose for containerization.
- [Postgres] - Postgres for database.

## Installation
Clone this repository
```sh
git clone https://github.com/bjergsen243/rust_hour
```

## Get started
Change to the project's folder
```sh
cd rust_hour
```
Build docker
```sh
docker compose build
```

Up the docker compose
```sh
docker compose up
```

Run the application
```sh
cargo run
```

Test the application
```sh
cargo test
```

## How to use
### Start the application
```sh
cargo run
```
### User
Create user

```sh
curl --location --request POST 'localhost:8080/registration' --header 'Content-Type: application/json' --data-raw '{"email": "example@gmail.com", "password": "123456789"}'
```

Sign in
```sh
curl --location --request POST 'localhost:8080/login' --header 'Content-Type: application/json' --data-raw '{"email": "example@gmail.com", "password": "123456789"}'
```
After sign in, you will have a `TOKEN`, you have to use it to do other actions.

Update user's email
```sh
curl --location --request PUT 'localhost:8080/accounts' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"email": "update@gmail.com"}'
```

Update user's password
```sh
curl --location --request PUT 'localhost:8080/accounts/update_password' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"password": "1234567890"}'
```

Get user's information
```sh
curl --location --request GET 'localhost:8080/accounts/me' --header 'Authorization: $TOKEN' 
```

### Question

Create question
```sh
curl --location --request POST 'localhost:8080/questions' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"title": "hello world", "content": "hello world", "tags": null}'
```

Update question
```sh
curl --location --request PUT 'localhost:8080/questions/1' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"id": 1, "title": "update title", "content": "update content", "tags": null}'
```

Delete question
```sh
curl --location --request DELETE 'localhost:8080/questions/1' --header 'Authorization: $TOKEN'
```

Get questions
```sh
curl --location --request GET 'localhost:8080/questions?limit=20&offset=0'
```

Get answers of question
```sh
curl --location --request GET 'localhost:8080/questions/1/answers?limit=20&offset=0' 
```

### Answer

Create answer
```sh
curl --location --request POST 'localhost:8080/answers' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"question_id": 2, "content": "answer question 2"}'

```

Update answer
```sh
curl --location --request PUT 'localhost:8080/answers/1' --header 'Authorization: $TOKEN' --header 'Content-Type: application/json' --data-raw '{"id": 1, "question_id": 2, "content": "update answer"}'

```

Delete answer
```sh
curl --location --request DELETE 'localhost:8080/answers/1' --header 'Authorization: $TOKEN'
```

## License

MIT

   [Rust]: <https://www.rust-lang.org/>
   [Docker & Docker Compose]: <https://docs.docker.com/get-started/>
   [Postgres]: <https://www.postgresql.org/>
   