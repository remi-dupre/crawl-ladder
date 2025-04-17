# Crawl Ladder

This is a quick project that aims at building a simple server that builds a
small ladder of which clients sends the most queries to it. This is only used
as a pedagogic tool for limited amount of clients and the simplicity of the
project comes with a few limitations :

- It can only scale vertically
- The ladder doesn't not persist across server restarts
- The metrics are overall quite naïve

## How to query

For a client, each request will require you to provide a token. Each token can
only be used once, otherwise they won't count in the metrics. Each valid
request will return a list of new valid tokens that you can use once.

For each request, the client must authenticate by providing a nickname in the
`X-User` header.

To get your first valid token, you first need to make a GET query to
`https://<BASE_URL>/crawl/`. This will answer with a list of tokens that you
can use for your next requests with a query to
`https://<BASE_URL>/crawl/<TOKEN>/`.
