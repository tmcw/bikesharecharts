use arrow::compute::sort;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};

use arrow_array::builder::PrimitiveBuilder;
use arrow_array::types::{GenericStringType, TimestampMillisecondType, UInt16Type, UInt64Type};
use arrow_array::{ArrayRef, RecordBatch};
use chrono::{NaiveDateTime, Timelike};
// use duckdb::{params, Connection};
// use duckdb::Connection;
use flate2::bufread;
use glob::glob;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

use serde::Deserialize;
use std::collections::HashMap;

use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct Station {
    #[serde(skip)]
    legacy_id: String,
    #[serde(skip)]
    last_reported: u64,
    num_ebikes_available: u16,
    num_bikes_available: u16,
    is_returning: u32,
    #[serde(skip)]
    eightd_has_available_keys: bool,
    num_docks_available: u16,
    #[serde(skip)]
    num_docks_disabled: u16,
    #[serde(skip)]
    is_installed: u32,
    num_bikes_disabled: u16,
    station_id: String,
    station_status: String,
    #[serde(skip)]
    is_renting: u32,
}

#[derive(Debug, Deserialize)]
struct Data {
    stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
struct StationStatus {
    data: Data,
    last_updated: i64,
    #[serde(skip)]
    ttl: u32,
}

struct Builders {
    id: u16,
    times: PrimitiveBuilder<TimestampMillisecondType>,
    station_ids: PrimitiveBuilder<UInt16Type>,
    num_bikes_available: PrimitiveBuilder<UInt16Type>,
    num_ebikes_available: PrimitiveBuilder<UInt16Type>,
    num_bikes_disabled: PrimitiveBuilder<UInt16Type>,
    num_docks_available: PrimitiveBuilder<UInt16Type>,
}

fn get_builders(id: u16) -> Builders {
    let mut times = PrimitiveBuilder::<TimestampMillisecondType>::new();
    let mut station_ids = PrimitiveBuilder::<UInt16Type>::new();
    let mut num_bikes_available = PrimitiveBuilder::<UInt16Type>::new();
    let mut num_ebikes_available = PrimitiveBuilder::<UInt16Type>::new();
    let mut num_bikes_disabled = PrimitiveBuilder::<UInt16Type>::new();
    let mut num_docks_available = PrimitiveBuilder::<UInt16Type>::new();

    return Builders {
        id,
        times,
        station_ids,
        num_bikes_available,
        num_ebikes_available,
        num_bikes_disabled,
        num_docks_available,
    };
}

fn main() {
    /*
        fs::remove_file("data.duckdb").unwrap();
        let conn = Connection::open("data.duckdb").unwrap();

        conn.execute(
            "CREATE TABLE station_status (
                    station_id VARCHAR,
                    num_bikes_available INTEGER,
                    num_ebikes_available INTEGER,
                    num_bikes_disabled INTEGER,
                    num_docks_available INTEGER,
                    time BIGINT
                )",
            [],
        )
        .unwrap();
    */

    let mut id_legend: HashMap<String, u16> = HashMap::new();
    let mut id_counter: u16 = 0;

    let mut batches: HashMap<String, Builders> = HashMap::new();

    for entry in glob("./station_status/*.json.gz").expect("Failed to read glob pattern") {
        // println!("Processing {:?}", entry);
        let input = BufReader::new(File::open(entry.unwrap()).unwrap());
        let mut decoder = bufread::GzDecoder::new(input);
        let status: StationStatus = serde_json::from_reader(&mut decoder).unwrap();
        let time = NaiveDateTime::from_timestamp_opt(status.last_updated, 0)
            .unwrap()
            .with_second(0)
            .unwrap();
        let stations: Vec<Station> = status
            .data
            .stations
            .into_iter()
            .filter(|station| station.station_status == "active")
            .collect();

        for station in &stations {
            let batch = batches
                .entry(station.station_id.clone().into())
                .or_insert_with(|| {
                    // This is partly because PrimitiveBuilder doesn't like strings.
                    // It would probably be nice to just use strings and let dictionary
                    // compression do its thing.
                    id_counter = id_counter + 1;
                    get_builders(id_counter)
                });
            batch.times.append_value(time.timestamp_millis());
            batch.station_ids.append_value(batch.id);
            batch
                .num_bikes_available
                .append_value(station.num_bikes_available - station.num_ebikes_available);
            batch
                .num_bikes_disabled
                .append_value(station.num_bikes_disabled);
            batch
                .num_ebikes_available
                .append_value(station.num_ebikes_available);
            batch
                .num_docks_available
                .append_value(station.num_docks_available);

            /*
                conn.execute(
                    "INSERT INTO station_status (
                            station_id,
                            num_bikes_available,
                            num_ebikes_available,
                            num_bikes_disabled,
                            num_docks_available,
                            time
                        ) VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        *station_id,
                        station.num_bikes_available,
                        station.num_ebikes_available,
                        station.num_bikes_disabled,
                        station.num_docks_available,
                        time.timestamp_millis(),
                    ],
                )
                .unwrap();
            */
        }
    }

    let file = File::create(format!("data.parquet")).unwrap();
    let station_ids = Field::new("station_ids", DataType::UInt16, false);
    let num_bikes_available = Field::new("num_bikes_available", DataType::UInt16, false);
    let num_ebikes_available = Field::new("num_ebikes_available", DataType::UInt16, false);
    let num_docks_available = Field::new("num_docks_available", DataType::UInt16, false);
    let num_bikes_disabled = Field::new("num_bikes_disabled", DataType::UInt16, false);
    let times_field = Field::new(
        "times",
        DataType::Timestamp(TimeUnit::Millisecond, None),
        false,
    );
    let schema = Schema::new(vec![
        station_ids,
        num_bikes_available,
        num_ebikes_available,
        num_bikes_disabled,
        num_docks_available,
        times_field,
    ]);
    let props = WriterProperties::builder();
    let mut writer = ArrowWriter::try_new(file, schema.into(), props.build().into()).unwrap();

    for mut builders in batches.into_values() {
        let batch = RecordBatch::try_from_iter(vec![
            (
                "station_ids",
                Arc::new(builders.station_ids.finish()) as ArrayRef,
            ),
            (
                "num_bikes_available",
                Arc::new(builders.num_bikes_available.finish()) as ArrayRef,
            ),
            (
                "num_ebikes_available",
                Arc::new(builders.num_ebikes_available.finish()) as ArrayRef,
            ),
            (
                "num_bikes_disabled",
                Arc::new(builders.num_bikes_disabled.finish()) as ArrayRef,
            ),
            (
                "num_docks_available",
                Arc::new(builders.num_docks_available.finish()) as ArrayRef,
            ),
            ("time", Arc::new(builders.times.finish()) as ArrayRef),
        ])
        .unwrap();

        // conn.close().unwrap();

        writer.write(&batch).expect("Writing batch");
    }

    // writer must be closed to write footer
    writer.close().unwrap();

    let mut file = File::create("id_map.json").unwrap();

    let serialized_data = serde_json::to_string(&id_legend).unwrap();
    file.write_all(serialized_data.as_bytes()).unwrap();
}
