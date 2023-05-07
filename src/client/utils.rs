use algorithm::proto::{player_service_client::PlayerServiceClient, PlayerRequest, WatchRoomRequest, Empty, HasWinnerRequest, HasWinnerResponse};
use anyhow::Result;
use tokio_stream::StreamExt;
use std::env;
use crate::{APP, BOARD};

pub async fn do_movement(room_id: String, command: String, shape: String) -> Result<String> {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "http://[::1]:50051".to_string());
    if command.trim().is_empty() {
        return Ok("empty movement".to_string());
    }
    let chordinates: Vec<&str> = command.trim().split(" ").collect();
    if chordinates.len() == 0 {
        return Ok("empty movement".to_string());
    }
    let mut client = PlayerServiceClient::connect(host)
        .await
        .unwrap();

    let x = chordinates[0].parse()?;
    let y = chordinates[1].parse()?;

    let request = tonic::Request::new(PlayerRequest {
        room_id: room_id.clone(),
        shape,
        x,
        y,
    });
    let response = client.do_movement(request).await.unwrap();
    let message = response.into_inner().message;
    Ok(message.clone())
}

pub async fn create_or_join_room() -> Result<()> {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "http://[::1]:50051".to_string());
    let mut app = APP.lock().await;
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!(
            "should pass at least a shape, can be any letter, i.e.: 'X' or 'O'"
        ));
    }
    let shape = args[1].clone();
    app.shape = shape;

    if args.len() == 3 {
        let room_id = args[2].clone();
        app.room_id = room_id;

        return Ok(());
    }
    let mut client = PlayerServiceClient::connect(host)
        .await
        .unwrap();

    let request = tonic::Request::new(Empty {});
    let response = client.create_room(request).await.unwrap();
    let room_id = response.into_inner().room_id;
    app.room_id = room_id;

    Ok(())
}

pub async fn loop_board_state(room_id: String) -> Result<HasWinnerResponse> {
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "http://[::1]:50051".to_string());
    let mut client = PlayerServiceClient::connect(host)
        .await
        .unwrap();

    let response = client
        .watch_room_stream(tonic::Request::new(WatchRoomRequest {
            room_id: room_id.clone(),
            spacing: 2,
        }))
        .await
        .unwrap();
    let mut stream = response.into_inner();

    // This method is a "infinite stream", but to match with the TUI main loop
    // we are changing to a simple pooling
    if let Some(item) = stream.next().await {
        let mut board = BOARD.lock().await;
        let item = item.unwrap();
        board.clear();
        board.push_str(item.message.as_str());
    }

    let response = client.has_winner(tonic::Request::new(HasWinnerRequest { room_id })).await?;
    Ok(response.into_inner())
}
