use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

pub async fn index() -> Html<&'static str> {
    Html(include_str!("../client/index.html"))
}

pub async fn style() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../client/style.css"),
    )
}

pub async fn app_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../client/app.js"),
    )
}

pub async fn scene_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../client/scene.js"),
    )
}

pub async fn combat_js() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        include_str!("../client/combat.js"),
    )
}

pub async fn icon(Path(name): Path<String>) -> impl IntoResponse {
    let name = name.strip_suffix(".svg").unwrap_or(&name);
    let body = match name {
        "anvil" => Some(include_str!("../client/icons/anvil.svg")),
        "arrow_stack" => Some(include_str!("../client/icons/arrow_stack.svg")),
        "auction" => Some(include_str!("../client/icons/auction.svg")),
        "blade" => Some(include_str!("../client/icons/blade.svg")),
        "boss" => Some(include_str!("../client/icons/boss.svg")),
        "bridge" => Some(include_str!("../client/icons/bridge.svg")),
        "cannon" => Some(include_str!("../client/icons/cannon.svg")),
        "chest" => Some(include_str!("../client/icons/chest.svg")),
        "crystal" => Some(include_str!("../client/icons/crystal.svg")),
        "currency" => Some(include_str!("../client/icons/currency.svg")),
        "deed" => Some(include_str!("../client/icons/deed.svg")),
        "gate" => Some(include_str!("../client/icons/gate.svg")),
        "gem" => Some(include_str!("../client/icons/gem.svg")),
        "guard" => Some(include_str!("../client/icons/guard.svg")),
        "hero" => Some(include_str!("../client/icons/hero.svg")),
        "key" => Some(include_str!("../client/icons/key.svg")),
        "mailbox" => Some(include_str!("../client/icons/mailbox.svg")),
        "monster" => Some(include_str!("../client/icons/monster.svg")),
        "mount" => Some(include_str!("../client/icons/mount.svg")),
        "npc" => Some(include_str!("../client/icons/npc.svg")),
        "potion" => Some(include_str!("../client/icons/potion.svg")),
        "quest_giver" => Some(include_str!("../client/icons/quest_giver.svg")),
        "relic" => Some(include_str!("../client/icons/relic.svg")),
        "scale" => Some(include_str!("../client/icons/scale.svg")),
        "shield" => Some(include_str!("../client/icons/shield.svg")),
        "shrine" => Some(include_str!("../client/icons/shrine.svg")),
        "trade_table" => Some(include_str!("../client/icons/trade_table.svg")),
        "wall" => Some(include_str!("../client/icons/wall.svg")),
        "wand" => Some(include_str!("../client/icons/wand.svg")),
        _ => None,
    };

    match body {
        Some(body) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")],
            body,
        ),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "icon not found",
        ),
    }
}
