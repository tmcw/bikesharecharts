# bikesharecharts worker

Cloudflare Worker for this project. This fetches New York Citibike `station_status.json`
every 5 minutes, gzips the JSON, and stashes it in an R2 bucket.
