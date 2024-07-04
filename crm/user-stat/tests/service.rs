use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use sqlx_db_tester::TestPg;
use tokio::time::sleep;
use tonic::transport::Server;

use user_stat::pb::{QueryRequestBuilder, RawQueryRequestBuilder};
use user_stat::pb::user_stats_client::UserStatsClient;
use user_stat::test_utils::{id, tq};
use user_stat::UserStatsService;

const PORT_BASE: u32 = 60000;

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats WHERE created_at > '2024-01-01' LIMIT 5")
        .build()?;

    let stream = client.raw_query(req).await?.into_inner();
    let ret = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert!(ret.len() > 0);
    Ok(())
}

#[tokio::test]
async fn query_should_work() -> Result<()> {
    let (_tdb, addr) = start_server(PORT_BASE + 1).await?;
    let mut client = UserStatsClient::connect(format!("http://{addr}")).await?;
    let req = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(Some(360), None)))
        .timestamp(("last_visited_at".to_string(), tq(Some(30), None)))
        .id(("viewed_but_not_started".to_string(), id(&[150174])))
        .build()
        .unwrap();

    let stream = client.query(req).await?.into_inner();
    let ret = stream
        .then(|res| async move { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(ret.len(), 0);
    Ok(())
}

async fn start_server(port: u32) -> Result<(TestPg, SocketAddr)> {
    let addr = format!("[::1]:{}", port).parse()?;

    let (tdb, svc) = UserStatsService::new_for_test().await?;
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc.into_server())
            .serve(addr)
            .await
            .unwrap()
    });
    sleep(Duration::from_micros(1)).await;

    Ok((tdb, addr))
}
