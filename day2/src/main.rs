use polars::{lazy::dsl::count, prelude::*};
use polars::error::PolarsResult;

fn main() -> PolarsResult<()> {
    let df = CsvReader::from_path("data/network_traffic.csv")
        .unwrap()
        .finish()?;

    //Task 1: Count the Packets
    let processed1 = df.clone().lazy().select([count()]).collect()?;

    let res1 = processed1["count"].iter().next().unwrap();

    println!("Number of packets in file: {}", res1);

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
    let res2 = processed2["Source"].iter().next().unwrap();

    println!("Most frequent sender: {}", res2.get_str().unwrap());

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

    let res3 = processed3["Protocol"].iter().next().unwrap();

    println!("Most frequent protocol: {}", res3.get_str().unwrap());
    Ok(())
}
