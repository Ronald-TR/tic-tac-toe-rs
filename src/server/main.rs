use algorithm::{
    proto::{
        player_service_server::{PlayerService, PlayerServiceServer},
        CreateRoomResponse, Empty, HasWinnerRequest, HasWinnerResponse, PlayerRequest,
        PlayerResponse, WatchRoomRequest,
    },
    tictactoe::{algorithm::TTTBoard, enums::PlayerShape},
};

use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use uuid::Uuid;

#[macro_use]
extern crate lazy_static;

use std::{collections::HashMap, str::FromStr};

lazy_static! {
    static ref HASHMAP: Mutex<HashMap<String, TTTBoard>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

#[derive(Debug, Default)]
struct MyPlayer {}

#[tonic::async_trait]
impl PlayerService for MyPlayer {
    async fn do_movement(
        &self,
        request: Request<PlayerRequest>,
    ) -> Result<Response<PlayerResponse>, Status> {
        let params = request.into_inner();
        let mut map = HASHMAP.lock().await;
        let board = map.get_mut(&params.room_id).unwrap();
        let player = PlayerShape::from_str(&params.shape).unwrap();
        board.insert(params.x as usize, params.y as usize, &player);
        let winner = board.find_winner();
        let mut message = String::from("Movement being done!! The game continues...");
        if winner.is_some() {
            message = format!(
                "We have a winner! {} is the winner",
                winner.unwrap().to_string()
            );
        };
        let reply = PlayerResponse {
            message: message.clone(),
        };
        println!("{}", message);
        Ok(Response::new(reply))
    }

    async fn create_room(&self, _: Request<Empty>) -> Result<Response<CreateRoomResponse>, Status> {
        let id = Uuid::new_v4();
        let mut board = TTTBoard::new();
        let mut map = HASHMAP.lock().await;

        board.players = 1;
        map.insert(id.to_string(), board);

        println!("ROOM ID: {}", id.to_string());
        Ok(Response::new(CreateRoomResponse {
            room_id: id.to_string(),
        }))
    }

    type WatchRoomStreamStream = ReceiverStream<Result<PlayerResponse, Status>>;

    async fn watch_room_stream(
        &self,
        request: Request<WatchRoomRequest>,
    ) -> Result<Response<Self::WatchRoomStreamStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let map = HASHMAP.lock().await;
        let params = request.into_inner();

        tokio::spawn(async move {
            let board = map.get(&params.room_id).unwrap();
            loop {
                let _ = match tx.try_send(Ok(PlayerResponse {
                    message: board.render_board(params.spacing as u8),
                })) {
                    Ok(_) => continue,
                    Err(_) => break,
                };
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn has_winner(
        &self,
        request: Request<HasWinnerRequest>,
    ) -> Result<Response<HasWinnerResponse>, Status> {
        let params = request.into_inner();
        let mut map = HASHMAP.lock().await;
        let board = map.get_mut(&params.room_id).unwrap();
        let winner = board.find_winner();

        if winner.is_some() {
            let reply = HasWinnerResponse {
                shape_winner: winner.unwrap().to_string(),
                has_winner: true,
            };
            return Ok(Response::new(reply));
        };
        Ok(Response::new(HasWinnerResponse {
            shape_winner: "".to_string(),
            has_winner: false,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let player = MyPlayer::default();
    let service = PlayerServiceServer::new(player);
    println!("Serving on {}", addr);
    Server::builder().add_service(service).serve(addr).await?;

    Ok(())
}
