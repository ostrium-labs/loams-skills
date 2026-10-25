# Company service image

Build the starter API as a container, then run it next to the company stack on
the tailnet. The service talks to Loams over the tailnet; it does not expose
Loams itself.

```sh
docker build -f deploy/company/Dockerfile -t company/api:dev .
docker run --rm -p 127.0.0.1:3000:3000 \
  -e LOAMS_ENDPOINT=http://loams:8080 \
  -e LOAMS_API_KEY=... \
  -e LOAMS_NAMESPACE=company \
  -e LOAMS_COLLECTION=notes \
  company/api:dev
```

- Bind the published port to loopback, or put the service behind the company
  gateway. Do not publish Loams.
- The image runs as uid 10001.
- Pass the API key as a secret, not in the image.
- For TLS, build `loams` with the `tls` feature (the starter already does when
  the endpoint is `https://`).
- Set `RUST_LOG=info` and ship logs to the company collector.
