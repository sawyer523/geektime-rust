use crm::pb::user_service_client::UserServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(crm::pb::GetUserRequest { id: 1 });
    let response = client.get_user(request).await?;
    println!("Response: {:?}", response.into_inner());
    Ok(())
}
