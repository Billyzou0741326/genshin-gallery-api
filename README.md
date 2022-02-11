# Genshin Gallery API

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
