# bikesharecharts

This is a project to look closely at bikeshare data, initially
in New York City, and see what we can find. I use the Citibike
system constantly, and notice all kinds of trends: fluctuating
ratios of electric to non-electric bicycles or certain docks that
are always full. The system is run by Lyft, which is [not
doing very well as a business](https://www.curbed.com/2023/04/lyft-bike-share-citi-bike.html), so the
system could transition to another company in the near future.

This is also a good opportunity to try out some new technology, like DuckDB
and Parquet. This project uses Parquet as a data format for the rides,
and DuckDB to load that data into a fast SQL database and drive the frontend.

There are three sub-projects in this repository:

- web, the DuckDB and SvelteKit-based frontend
- collector, the Rust-based tool that parses and compresses ride data into Parquet files
- worker, the script that requests the latest station_status information every five minutes and stores it in R2

---

This project is a very early work in progress, and something I'm
doing in my limited free time. If you want to contribute on an
existing issue or propose something that you might want to do,
go for it! I'd love to collaborate with folks.
