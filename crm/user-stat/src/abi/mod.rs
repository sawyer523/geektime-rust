use chrono::{DateTime, TimeZone, Utc};
use futures::stream;
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{ResponseStream, ServiceResult, UserStatsService};
use crate::pb::{QueryRequest, RawQueryRequest, User};

impl UserStatsService {
    pub async fn query(&self, req: QueryRequest) -> ServiceResult<ResponseStream> {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = req
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.lower, v.upper))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let ids_conditions = req
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .join(" AND ");

        sql.push_str(" AND ");
        sql.push_str(&ids_conditions);

        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.inner.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query: {}",
                req.query
            )));
        };
        Ok(Response::new(Box::pin(stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, before: Option<Timestamp>, after: Option<Timestamp>) -> String {
    if before.is_none() && after.is_none() {
        return "".to_string();
    }

    if before.is_none() {
        let after = ts_to_utc(after.unwrap());
        return format!("{} <= '{}'", name, after);
    }

    if after.is_none() {
        let before = ts_to_utc(before.unwrap());
        return format!("{} >= '{}'", name, before);
    }

    let before = ts_to_utc(before.unwrap());
    let after = ts_to_utc(after.unwrap());
    format!("{} BETWEEN '{}' AND '{}'", name, before, after)
}

fn ts_to_utc(after: Timestamp) -> DateTime<Utc> {
    let after = Utc.timestamp_opt(after.seconds, after.nanos as _).unwrap();
    after
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use futures::StreamExt;

    use crate::AppConfig;
    use crate::pb::{IdQuery, QueryRequestBuilder, TimeQuery};

    use super::*;

    #[tokio::test]
    async fn raw_query_should_word() -> Result<()> {
        let config = AppConfig::load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;
        let mut stream = svc
            .raw_query(RawQueryRequest {
                query: "SELECT email, name FROM user_stats WHERE created_at > '2024-01-01' limit 5"
                    .to_string(),
            })
            .await?
            .into_inner();

        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }

        Ok(())
    }

    #[tokio::test]
    async fn query_should_work() -> Result<()> {
        let config = AppConfig::load().expect("Failed to load config");
        let svc = UserStatsService::new(config).await;
        let req = QueryRequestBuilder::default()
            .timestamp(("created_at".to_string(), tq(Some(120), None)))
            .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
            .id(("viewed_but_not_started".to_string(), id(&[252790])))
            .build()
            .unwrap();

        let mut stream = svc.query(req).await?.into_inner();
        while let Some(res) = stream.next().await {
            println!("{:?}", res);
        }
        Ok(())
    }

    fn id(id: &[u32]) -> IdQuery {
        IdQuery { ids: id.to_vec() }
    }

    fn tq(lower: Option<i64>, upper: Option<i64>) -> TimeQuery {
        TimeQuery {
            lower: lower.map(to_ts),
            upper: upper.map(to_ts),
        }
    }

    fn to_ts(days: i64) -> Timestamp {
        let dt = Utc::now()
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap();
        Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as _,
        }
    }
}
