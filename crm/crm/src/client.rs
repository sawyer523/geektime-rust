use anyhow::Result;
use tonic::metadata::MetadataValue;
use tonic::Request;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use uuid::Uuid;

use crm::pb::crm_client::CrmClient;
use crm::pb::RemindRequestBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let pem = include_str!("../../fixtures/rootCA.pem");
    let tls = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("localhost");
    let channel = Channel::from_static("https://[::1]:50000")
        .tls_config(tls)?
        .connect()
        .await?;

    let token = include_str!("../../fixtures/token").trim();
    let token: MetadataValue<_> = format!("Bearer {}", token).parse()?;

    let mut client = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    // let req = WelcomeRequestBuilder::default()
    //     .id(Uuid::new_v4().to_string())
    //     .interval(99u32)
    //     .content_ids([1u32, 2, 3])
    //     .build()?;
    //
    // let response = client
    //     .clone()
    //     .welcome(Request::new(req))
    //     .await?
    //     .into_inner();
    // println!("Response: {:?}", response);
    //
    // let req = RecallRequestBuilder::default()
    //     .id(Uuid::new_v4().to_string())
    //     .last_visit_interval(8u32)
    //     .content_ids([1u32, 2, 3])
    //     .build()?;
    //
    // let response = client.clone().recall(Request::new(req)).await?.into_inner();
    // println!("Response: {:?}", response);

    let req = RemindRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .last_visit_interval(8u32)
        .build()?;
    let response = client.remind(Request::new(req)).await?.into_inner();

    println!("Response: {:?}", response);
    Ok(())
}
