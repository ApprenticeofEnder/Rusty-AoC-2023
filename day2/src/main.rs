use polars::error::PolarsResult;
use polars::{lazy::dsl::count, prelude::*};

fn main() -> PolarsResult<()> {
    let df = CsvReader::from_path("data/network_traffic.csv")?.finish()?;

    //Task 1: Count the Packets
    let processed1 = df.clone().lazy().select([count()]).collect()?;

    if let Some(res1) = processed1["count"].iter().next() {
        println!("Number of packets in file: {}", res1);
    } else {
        println!("Something went wrong with task 1.");
    }

    //Task 2
    let processed2 = df
        .clone()
        .lazy()
        .group_by(["Source"])
        .agg([count()])
        .sort(
            "count",
            SortOptions {
                descending: true,
                nulls_last: true,
                ..Default::default()
            },
        )
        .limit(1)
        .collect()?;

    if let Some(res2) = processed2["Source"].iter().next() {
        println!("Most frequent sender: {}", res2.get_str().unwrap());
    } else {
        println!("Something went wrong with task 2.");
    }

    //Task 3
    let processed3 = df
        .clone()
        .lazy()
        .group_by(["Protocol"])
        .agg([count()])
        .sort(
            "count",
            SortOptions {
                descending: true,
                nulls_last: true,
                ..Default::default()
            },
        )
        .limit(1)
        .collect()?;

    if let Some(res3) = processed3["Protocol"].iter().next() {
        println!("Most frequent protocol: {}", res3.get_str().unwrap());
    } else {
        println!("Something went wrong with task 3.");
    }
    Ok(())
}
