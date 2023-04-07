use warp::Filter;

#[tokio::main]
async fn main() {
    let get_players = warp::get()
        .and(warp::path("players"))
        .and(warp::path::end())
        .and_then(get_players);

    let get_player = warp::get()
        .and(warp::path("players"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and_then(get_player);

    let post_player = warp::post()
        .and(warp::path("players"))
        .and(warp::path::end())
        .and_then(post_player);

    let put_player = warp::put()
        .and(warp::path("players"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and_then(put_player);

    let delete_player = warp::delete()
        .and(warp::path("players"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and_then(delete_player);

    let routes = get_players
        .or(get_player)
        .or(post_player)
        .or(put_player)
        .or(delete_player);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn get_players() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Get players"))
}

async fn get_player(id: u32) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Get players by id {}", id))
}

async fn post_player() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Post players"))
}

async fn put_player(id: u32) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Put players by id {}", id))
}

async fn delete_player(id: u32) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("Delete players by id {}", id))
}
