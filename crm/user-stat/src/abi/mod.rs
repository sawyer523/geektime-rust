use std::fmt;
use std::fmt::Formatter;

use chrono::{DateTime, TimeZone, Utc};
use futures::stream;
use itertools::Itertools;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{ResponseStream, ServiceResult, UserStatsService};
use crate::pb::{
    QueryRequest, QueryRequestBuilder, RawQueryRequest, TimeQuery, User,
};

impl UserStatsService {
    pub async fn query(&self, req: QueryRequest) -> ServiceResult<ResponseStream> {
        let sql = req.to_string();
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

impl QueryRequest {
    pub fn new_with_dt(name: &str, lower: DateTime<Utc>, upper: DateTime<Utc>) -> Self {
        let ts = Timestamp {
            seconds: lower.timestamp(),
            nanos: 0,
        };
        let ts1 = Timestamp {
            seconds: upper.timestamp(),
            nanos: 0,
        };
        let tq = TimeQuery {
            lower: Some(ts),
            upper: Some(ts1),
        };

        QueryRequestBuilder::default()
            .timestamp((name.to_string(), tq))
            .build()
            .expect("Failed to build query request")
    }
}

impl fmt::Display for QueryRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut sql = "SELECT email, name FROM user_stats WHERE ".to_string();

        let time_conditions = self
            .timestamps
            .iter()
            .map(|(k, v)| timestamp_query(&k, v.lower.as_ref(), v.upper.as_ref()))
            .join(" AND ");

        sql.push_str(&time_conditions);

        let ids_conditions = self
            .ids
            .iter()
            .map(|(k, v)| ids_query(&k, v.ids.as_ref()))
            .join(" AND ");

        if !ids_conditions.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&ids_conditions);
        }

        write!(f, "{}", sql)
    }
}

fn ids_query(name: &str, ids: &[u32]) -> String {
    if ids.is_empty() {
        return "TRUE".to_string();
    }

    format!("array{:?} <@ {}", ids, name)
}

fn timestamp_query(name: &str, before: Option<&Timestamp>, after: Option<&Timestamp>) -> String {
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

    let before = ts_to_utc(before.unwrap()).to_rfc3339();
    let after = ts_to_utc(after.unwrap()).to_rfc3339();
    format!("{} BETWEEN '{}' AND '{}'", name, before, after)
}

fn ts_to_utc(after: &Timestamp) -> DateTime<Utc> {
    let after = Utc.timestamp_opt(after.seconds, after.nanos as _).unwrap();
    after
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use futures::StreamExt;

    use crate::pb::QueryRequestBuilder;
    use crate::test_utils::{id, tq};

    use super::*;

    #[tokio::test]
    async fn query_request_to_string_should_work() -> Result<()> {
        let d1 = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
        let query = QueryRequest::new_with_dt("created_at", d1, d2);
        let sql = query.to_string();
        assert_eq!(
            sql,
            "SELECT email, name FROM user_stats WHERE created_at BETWEEN '2024-01-01T00:00:00+00:00' AND '2024-01-02T00:00:00+00:00'"
        );
        Ok(())
    }

    #[tokio::test]
    async fn raw_query_should_work() -> Result<()> {
        let (_tdb, svc) = UserStatsService::new_for_test().await?;
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
        let (_tdb, svc) = UserStatsService::new_for_test().await?;
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
}
