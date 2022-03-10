# Genshin Gallery API

## Grading guide

A disposable mongodb instance is provided. Simply build and run the API server.
The server will listen on port 8000 (by default):

```shell
$ cargo build --release
$ export MONGODB_URL=mongodb://dev:w22rust_410510_sample@34.83.92.78/pixiv?authSource=admin
$ target/release/genshin-gallery-api
```

Powershell has a different way of specifying environment variables:

```powershell
PS > cargo build --release
PS > $Env:MONGODB_URL = "mongodb://dev:w22rust_410510_sample@34.83.92.78/pixiv?authSource=admin"
PS > .\target\release\genshin-gallery-api.exe
```

On <https://w22rust.minamiktr.com> swagger-ui is deployed for API documentation and testing. 
From the `Servers` dropdown, select `http://localhost:8000/` and start testing APIs.

Under each API, `Try it out` and `Execute` the example.

Meanwhile on the console running the API server, there should be logs showing API requests.

Otherwise no GUIs or web interface is provided for this project. This is a standalone project
unrelated to the live deployment, and while that web frontend uses this rust implementation
as its backend, the web interface is not in the scope of this project and is not for grading.

Care was taken to avoid copyrights violation. See below for details.


## Copyright concerns (pixiv)

Pixiv has no explicit policies to prohibit reposting. It is up to individual
artists to decide whether they allow the reposting / redistribution of their artwork.

List of pixiv artists that ok'd reposting as long as source to the art is provided:

| pixiv id     | username           | Repost w/ source specified w/o permission  |
|--------------|--------------------|--------------------------------------------|
| 13772078     | 夜蓝星炫            | Yes                                        |
| 4588267      | Fukuro袋子          | Yes (paid content not allowed)            |
| 59375709     | Torriet-トリエット- | Yes                                        |
| 13146300     | 红白0v0             | Yes (no commercial use)                   |
| 8893081      | 绫纱岚              | No (only personal use)                    |
| 13771077     | 木吉子              | Yes                                       |
| 7321921      | Nami                | Yes (no commercial use)                   |
| 4346254      | Vardan              | Yes                                       |
| 5302293      | 雲こ猫（雲小猫）     | Yes                                       |
| 58434088     | 喵咕君QAQ(KH3)      | Yes                                       |
| 6657532      | QuAn_               | Yes                                       |
| 20060518     | い酢ず              | Yes (twitter @an_icy_cat)                 |
| 1513922      | あいに              | Yes                                       |
| 13679278     | amco                | Yes                                       |
| 20033303     | 羽蛇Quetzalli       | Yes (no commercial use)                   |
| 36696168     | 早栖Maki            | Yes (no commercial use)                   |
| 14475287     | 風遠                | Yes (no commercial use)                   |
| 3420822      | Csyday              | Yes (no commercial use, no edits)         |
| 15770844     | Those Girls         | Yes                                       |
| 60444940     | 鷹なゆ              | No (only personal use)                    |
| 4325078      | アイスワイン@icwine  | Yes                                       |
| 65309246     | 霓缎の舞            | Yes (no commercial use)                   |
| 28598181     | Chernyyvo           | Yes (no commercial use)                   |
| 7637225      | 核核hzb             | Yes (no commercial use)                   |
| 8520686      | Ikazu401            | Yes (no commercial use)                   |
| 4959310      | 桃花                | Yes (no commercial use)                   |
| 41412923     | ene                 | Yes (no commercial use)                   |
| 15205988     | 貢 茶               | Yes                                       |
| 13072125     | Sakon04             | Yes (no commercial use)                   |
| 3416394      | ゆりりん            | Yes (no commercial use)                   |
| 30716447     | ginklaga            | Yes (no commercial use)                   |
| 16293449     | Arden               | Yes (no commercial use)                   |
| 4460847      | Criin               | Yes (no commercial use)                   |
| 34661473     | 宜o0                | Yes (no commercial use)                   |
| 5004458      | ネイト二世          | Yes (no commercial use)                   |
| 8848483      | ラーレ              | Yes                                       |
| 8321385      | Shotz              | Yes (no commercial use)                   |
| 21029246     | 汉堡包Hbb_5412      | Yes (no commercial use)                   |

## Copyright concerns (Genshin Impact | mihoyo)

Mihoyo encourages the creation of fan arts:

> Travelers (players) are free to create original secondary content 
> from already published content of the Genshin Impact series and 
> produce physical merchandise to give away or sell. However,
> the production of physical merchandise is subject to the 
> following conditions:
>
> I. Fanart used for the production of unofficial merchandise 
> must not violate relevant laws and regulations under any 
> circumstances; nor shall they infringe on the rights and
> interests of third parties, the Genshin Impact development
> team, and miHoYo. Furthermore they shall not harm the reputation
> of Genshin Impact or miHoYo.
> 
> II. Fan art creators may not register the relevant content
> of the creation as a trademark in any country or region,
> nor may they register copyrights, patents, etc. for fan
> art and/or fan-made merchandise in any country or region.

Source taken from the [official policy](https://www.hoyolab.com/article/381519)
from mihoyo regarding fanart and merchadise.


## Disclaimer

Artworks collected in the database **will not be used for commercial purposes**.

**No images are modified, downloaded, or saved**. Only hyperlinks to the source artwork are stored.
If the artist chose to remove their artwork from the source platform ([pixiv](https://pixiv.net/)), 
the hyperlinks to the corresponding artwork will yield 404 Not Found, and the database will mark 
them as `is_404` and may not display said artwork.

In addition, only artworks blessed by their artists for reposting will be used in the submission.


## Deployment

Docker / kubernetes is the recommended way of deploying the project.

```shell
MONGODB_URL=mongodb://127.0.0.1/pixiv?authSource=admin genshin-gallery-api 
```

or use `dotenv` to simplify loading environment variables

```shell
dotenv genshin-gallery-api
```

or use Docker

```shell
docker run --rm -p 8000:8000 -e MONGODB_URL=mongodb://127.0.0.1/pixiv?authSource=admin museaqours/genshin-gallery-api:latest
```

## Mongodb dependency

Spin up a disposable mongodb instance via docker (for dev and testing, not for deployment):

```shell
docker run -d \
  --name test-mongo \
  -p 127.0.0.1:27017:27017 \
  -e MONGO_INITDB_ROOT_USERNAME=dev \
  -e MONGO_INITDB_ROOT_PASSWORD=dev \
  -e MONGO_INITDB_DATABASE=pixiv \
  mongo:4.4.12-rc1
```

For deployment it is recommended to use a mongodb cluster, managed or self-hosted.
