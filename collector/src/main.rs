use arrow::compute::sort;
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};

use arrow_array::builder::PrimitiveBuilder;
use arrow_array::types::{Int32Type, TimestampSecondType};
use arrow_array::{ArrayRef, RecordBatch};
use chrono::{NaiveDateTime, Timelike};
use flate2::bufread;
use glob::{glob, GlobError};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

use indicatif::ProgressBar;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::sync::Arc;

type IntyBaseType = i32;

#[derive(Debug, Deserialize)]
struct Station {
    #[serde(skip)]
    legacy_id: String,
    #[serde(skip)]
    last_reported: u64,
    num_ebikes_available: IntyBaseType,
    num_bikes_available: IntyBaseType,
    is_returning: u32,
    #[serde(skip)]
    eightd_has_available_keys: bool,
    num_docks_available: IntyBaseType,
    #[serde(skip)]
    num_docks_disabled: IntyBaseType,
    #[serde(skip)]
    is_installed: u32,
    num_bikes_disabled: IntyBaseType,
    station_id: String,
    station_status: Option<String>,
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

type IntyType = Int32Type;

type TimestampType = TimestampSecondType;

#[derive(Debug)]
struct Builders {
    id: IntyBaseType,
    times: PrimitiveBuilder<TimestampType>,
    station_ids: PrimitiveBuilder<IntyType>,
    num_bikes_available: PrimitiveBuilder<IntyType>,
    num_ebikes_available: PrimitiveBuilder<IntyType>,
    num_bikes_disabled: PrimitiveBuilder<IntyType>,
    num_docks_available: PrimitiveBuilder<IntyType>,
}

fn get_builders(id: IntyBaseType) -> Builders {
    let times = PrimitiveBuilder::<TimestampType>::new();
    let station_ids = PrimitiveBuilder::<IntyType>::new();
    let num_bikes_available = PrimitiveBuilder::<IntyType>::new();
    let num_ebikes_available = PrimitiveBuilder::<IntyType>::new();
    let num_bikes_disabled = PrimitiveBuilder::<IntyType>::new();
    let num_docks_available = PrimitiveBuilder::<IntyType>::new();

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
    let id_legend: HashMap<String, IntyBaseType> = HashMap::new();
    let mut id_counter: IntyBaseType = 0;

    let mut batches: HashMap<String, Builders> = HashMap::new();

    let files = glob("./station_status/*.json.gz").expect("Failed to read glob pattern");

    let paths: Vec<Result<PathBuf, GlobError>> = files.collect();

    let bar = ProgressBar::new(paths.len() as u64);

    for entry in paths {
        let unwrapped_entry = entry.unwrap();
        // println!("Processing {:?}", entry);
        let input = BufReader::new(File::open(&unwrapped_entry).unwrap());
        let mut decoder = bufread::GzDecoder::new(input);
        let status_result: Result<StationStatus, serde_json::Error> =
            serde_json::from_reader(&mut decoder);

        match status_result {
            Ok(status) => {
                let time = NaiveDateTime::from_timestamp_opt(status.last_updated, 0)
                    .unwrap()
                    .with_second(0)
                    .unwrap();

                // Only collect hourly
                let stations: Vec<Station> = status.data.stations.into_iter().collect();

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
                }
            }
            Err(error) => {
                println!(
                    "Bad file: {}, error: {}",
                    unwrapped_entry.to_str().unwrap(),
                    error
                );
            }
        }
        bar.inc(1);
    }

    let inty_data_type = DataType::Int32;

    for (id, mut builders) in batches {
        let station_ids = Field::new("station_ids", inty_data_type.clone(), false);
        let num_bikes_available = Field::new("num_bikes_available", inty_data_type.clone(), false);
        let num_ebikes_available =
            Field::new("num_ebikes_available", inty_data_type.clone(), false);
        let num_docks_available = Field::new("num_docks_available", inty_data_type.clone(), false);
        let num_bikes_disabled = Field::new("num_bikes_disabled", inty_data_type.clone(), false);
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
        let props = WriterProperties::builder()
            .set_compression(parquet::basic::Compression::SNAPPY)
            .set_writer_version(parquet::file::properties::WriterVersion::PARQUET_2_0);
        let file = File::create(format!("./output/station-{id}.parquet")).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema.into(), Some(props.build())).unwrap();
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

        writer.write(&batch).expect("Writing batch");
        // writer must be closed to write footer
        writer.close().unwrap();
    }

    let mut file = File::create("id_map.json").unwrap();
    let serialized_data = serde_json::to_string(&id_legend).unwrap();
    file.write_all(serialized_data.as_bytes()).unwrap();
}
