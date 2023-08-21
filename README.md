# Rate limiter system design (WIP

```
                            ┌──────────┐
                            │          │
                            │  redis   ├──────────────┐
                            │          │              │
                            └────▲─────┘              │
                                 │                    │
                                 │                    │
┌──────────────┐                 │            ┌───────▼────────┐
│              │            ┌────┴─────┐      │                │
│              │            │          │      │                │
│    Client    ├───────────►│middleware├──────►   Web Server   │
│              │            │          │      │                │
│              │            └──────────┘      │                │
└───────▲──────┘                              └───────┬────────┘
        │                                             │
        └─────────────────────────────────────────────┘
```

- Client makes request to webserver
- Middleware layer will intercept the request object and increment "host":"req_count" in redis
- middleware layer will tell webserver to reuturn 200 OK until limit is reached
- if limit is reached webserver will respond with 429 TOO MANY REQUESTS, "Too many requests" in body and the following headers:
    - "X-RateLimit-Limit"
    - "X-RateLimit-Remaining"
    - "X-RateLimit-Reset"