use model::rooms::Room;

pub fn fmt_room(room: Room) -> &'static str {
    match room {
        Room::Adult => "🧘 Взрослые",
        Room::Child => "🧒 Дети",
    }
}

pub fn fmt_room_emoji(room: Room) -> &'static str {
    match room {
        Room::Adult => "🧘",
        Room::Child => "🧒",
    }
}