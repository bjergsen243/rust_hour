### rust_hour


Build docker image:
```bash
docker compose build
```
Up docker compose
```bash
docker compose up
```
Access db
``` bash
psql postgres://sotatek:1@localhost:5432/rustwebdev
```

Access postgres through docker
```bash
docker exec -it CONTAINER_ID
```
After that run
``` bash
psql postgres://sotatek:1@localhost:5432/rustwebdev
```


registration:
```bash
curl --location --request POST 'localhost:8080/registration' --header 'Content-Type: application/json' --data-raw '{"email": "example@gmail.com", "password": "123456789"}'
```
login:
```bash
curl --location --request POST 'localhost:8080/login' --header 'Content-Type: application/json' --data-raw '{"email": "example@gmail.com", "password": "123456789"}'
```
-> token: 
v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo


create question
```bash

curl --location --request POST 'localhost:8080/questions' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"title": "hello world", "content": "hello world", "tags": null}'

```

Get question:
```bash

curl --location --request GET 'localhost:8080/questions?limit=20&offset=0'
```
Update question:
```bash

curl --location --request PUT 'localhost:8080/questions/1' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"id": 1, "title": "update title", "content": "update content", "tags": null}'
```
Delete question:
```bash

curl --location --request DELETE 'localhost:8080/questions/1' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo'
```

Add answer:
```bash

curl --location --request POST 'localhost:8080/answers' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"question_id": 2, "content": "answer question 2"}'
```

Get answer of question:
```bash

curl --location --request GET 'localhost:8080/questions/2/answers?limit=20&offset=0' 
````
Update answer
```bash

curl --location --request PUT 'localhost:8080/answers/1' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"id": 1, "question_id": 2, "content": "update answer"}'
```
Delete answer:

```bash

curl --location --request DELETE 'localhost:8080/answers/1' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo'
```

- Update user email
```bash

curl --location --request PUT 'localhost:8080/accounts' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"email": "update@gmail.com"}'
```
- Get user information
```bash

curl --location --request GET 'localhost:8080/accounts/me' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' 
```
- Update password
```bash

curl --location --request PUT 'localhost:8080/accounts/update_password' --header 'Authorization: v2.local.SFUXOn8pw_GDRv486iCNOlODcrIX_eEZ8sHSDzllSzOV5RuxXqsjFaGhG3aTRLOHpLAAiFPxVAt1YbQHUtiz3Y9XS7KcDXjG2UlvMVEbdRrlXlJ05Y5Wsf1Vy9rPzStISV1mZHW8i2_se1uPr6gGMMWR5dKePWgiWSLU6DlMsaVtsMHfKOo' --header 'Content-Type: application/json' --data-raw '{"password": "1234567890"}'
```