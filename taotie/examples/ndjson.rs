use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use anyhow::Result;
use arrow::array::AsArray;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::json::ReaderBuilder;

fn main() -> Result<()> {
    /*
        ┌─name─────────────────────┬─type───────────────────────────┬─default_type─┬─default_expression─┬─comment─┬─codec_expression─┬─ttl_expression─
     1. │ email                    │ Nullable(String)               │              │                    │         │                  │                │
     2. │ name                     │ Nullable(String)               │              │                    │         │                  │                │
     3. │ gender                   │ Nullable(String)               │              │                    │         │                  │                │
     4. │ created_at               │ Nullable(DateTime64(6, 'UTC')) │              │                    │         │                  │                │
     5. │ last_visited_at          │ Nullable(DateTime64(6, 'UTC')) │              │                    │         │                  │                │
     6. │ last_watched_at          │ Nullable(DateTime64(6, 'UTC')) │              │                    │         │                  │                │
     7. │ recent_watched           │ Array(Nullable(Int32))         │              │                    │         │                  │                │
     8. │ viewed_but_not_started   │ Array(Nullable(Int32))         │              │                    │         │                  │                │
     9. │ started_but_not_finished │ Array(Nullable(Int32))         │              │                    │         │                  │                │
    10. │ finished                 │ Array(Nullable(Int32))         │              │                    │         │                  │                │
    11. │ last_email_notification  │ Nullable(DateTime64(6, 'UTC')) │              │                    │         │                  │                │
    12. │ last_in_app_notification │ Nullable(DateTime64(6, 'UTC')) │              │                    │         │                  │                │
    13. │ last_sms_notification    │ Nullable(DateTime64(6, 'UTC')) │
         */

    let schema = Schema::new(vec![
        Field::new("email", DataType::Utf8, true),
        Field::new("name", DataType::Utf8, true),
        Field::new("gender", DataType::Utf8, true),
        Field::new("created_at", DataType::Date64, true),
        Field::new("last_visited_at", DataType::Date64, true),
        Field::new("last_watched_at", DataType::Date64, true),
        Field::new(
            "recent_watched",
            DataType::List(Arc::new(Field::new(
                "recent_watched",
                DataType::Int32,
                true,
            ))),
            true,
        ),
        Field::new(
            "viewed_but_not_started",
            DataType::List(Arc::new(Field::new(
                "viewed_but_not_started",
                DataType::Int32,
                true,
            ))),
            true,
        ),
        Field::new(
            "started_but_not_finished",
            DataType::List(Arc::new(Field::new(
                "started_but_not_finished",
                DataType::Int32,
                true,
            ))),
            true,
        ),
        Field::new(
            "finished",
            DataType::List(Arc::new(Field::new("finished", DataType::Int32, true))),
            true,
        ),
        Field::new("last_email_notification", DataType::Date64, true),
        Field::new("last_in_app_notification", DataType::Date64, true),
        Field::new("last_sms_notification", DataType::Date64, true),
    ]);

    // load data from ndjson file
    let file = BufReader::new(File::open("assets/users.ndjson")?);
    let reader = ReaderBuilder::new(Arc::new(schema)).build(file)?;
    for batch in reader {
        let batch = batch?;
        println!("{:?}", batch.slice(0, 1).columns());
        let email = batch.column(0).as_string::<i32>();
        let name = batch.column(1).as_string::<i32>();
        println!("{} {}", email.value(0), name.value(0));
    }

    Ok(())
}
