# notion2ics

Create a subscribable calendar from a Notion database 

> **⚠️ Note:** This project is still in early development, expect bugs / missing features.

---

**Example**: A database entry from Notion website:

<img src="https://user-images.githubusercontent.com/6001248/227716980-4fe36f80-a378-469a-8372-9af76c6cdbc4.png" width="600">

and the same entry as calendar event (Thunderbird):

![image](https://user-images.githubusercontent.com/6001248/227716905-9e6784bf-b95a-470c-91fb-eeca9f4429b3.png)


## Usage

Required information:
- `api-token`: The API token of a Notion integration. [Learn More](https://www.notion.so/my-integrations)
- `database`: The ID can be retrieved from the database URL. [Learn More](https://developers.notion.com/reference/retrieve-a-database)

**Note:** Don't forget to add your created Notion integration to your workspace / to your sites (using top right "..." menu -> Integrations)

### Command Line

```
notion2ics --api-token secret_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX \
           --database 1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
           --database 2xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx \
           --refresh-intervall 15min \
           --output-path /html/calendar/
```

### Container

```
version: "3"
services:
  notion2ics:
    image: ghcr.io/haecker-felix/notion2ics:latest
    volumes:
      - /path/to/html/root/:/html
    command:
      - "--api-token=secret_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
      - "--database=1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
      - "--database=2xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
      - "--refresh-intervall=15min"
      - "--output-path=/html/calendar/"
```
