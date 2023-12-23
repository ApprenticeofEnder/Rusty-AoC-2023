use polars::error::PolarsResult;
use polars::{lazy::dsl::count, prelude::*};

fn most_popular<'a>(column: &'a str, df: &'a DataFrame) -> PolarsResult<DataFrame> {
    Ok(df
        .clone()
        .lazy()
        .group_by([column])
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
        .collect()?)
}

fn main() -> PolarsResult<()> {
    let df: DataFrame = CsvReader::from_path("data/network_traffic.csv")?.finish()?;

    //Task 1: Count the Packets
    let processed1: DataFrame = df.clone().lazy().select([count()]).collect()?;

    if let Some(res1) = processed1["count"].iter().next() {
        println!("Number of packets in file: {}", res1);
    } else {
        println!("Something went wrong with task 1.");
    }

    //Task 2: IP Address with the most packets sent
    let processed2: DataFrame = most_popular("Source", &df)?;

    if let Some(res2) = processed2["Source"].iter().next() {
        println!("Most frequent sender: {}", res2.get_str().unwrap());
    } else {
        println!("Something went wrong with task 2.");
    }

    //Task 3: Most Popular Protocol
    let processed3: DataFrame = most_popular("Protocol", &df)?;

    if let Some(res3) = processed3["Protocol"].iter().next() {
        println!("Most frequent protocol: {}", res3.get_str().unwrap());
    } else {
        println!("Something went wrong with task 3.");
    }
    Ok(())
}
