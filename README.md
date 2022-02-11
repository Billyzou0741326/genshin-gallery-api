# Genshin Gallery API

## Deployment

```shell
MONGODB_URL=mongodb://127.0.0.1/pixiv?authSource=admin genshin-gallery-api 
```

or use `dotenv` to simplify loading environment variables

```shell
dotenv genshin-gallery-api
```

or use Docker

```shell
docker run --rm -p 8000:8000 -e MONGODB_URL=mongodb://127.0.0.1/pixiv?authSource=admin genshin-gallery-api:latest
```

## Mongodb dependency

```shell
docker run -d \
  --name test-mongo \
  -p 127.0.0.1:27017:27017 \
  -e MONGO_INITDB_ROOT_USERNAME=dev \
  -e MONGO_INITDB_ROOT_PASSWORD=dev \
  -e MONGO_INITDB_DATABASE=pixiv \
  mongo:4.4.12-rc1
```
