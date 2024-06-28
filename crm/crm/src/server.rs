use crm::pb::user_service_server::{UserService, UserServiceServer};
use crm::pb::{CreateUserRequest, GetUserRequest, User};
use tonic::transport::Server;
use tonic::{async_trait, Request, Response, Status};

#[derive(Default)]
pub struct UserServer {}

#[async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("get_user request: {:?}", input);
        Ok(Response::new(User::new(1, "Alice", "alice@acme.com")))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("get_user request: {:?}", input);
        Ok(Response::new(User::new(1, "Alice", "alice@acme.com")))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = "[::1]:50051".parse()?;
    let svc = UserServer::default();

    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
