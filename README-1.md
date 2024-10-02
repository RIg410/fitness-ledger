
Промпт тест

Сгенерируй мне структуру данных отражающую запросы пользователя к сервису фитнес клуба.
доступны следующие типы запросов
показать список тренеровок (по программе, инструктора, дню недели)
показать мои тренировки
отменить запись на тренировку
записаться по времени  
pub enum RequestType {
    ShowWorkouts { program: Option<String>, instructor: Option<String>, day_of_week: Option<String> },
    ShowMyWorkouts,
    CancelWorkout { workout_id: u32 },
    BookWorkout { time: String },
}

pub struct UserRequest {
    pub user_id: u32,
    pub request_type: RequestType,
}
Ты помошник в фитнес клубе.
надо превращать текстовые запросы пользователей в запросы к api
у api есть такие варианты
pub enum RequestType {
    ShowWorkouts { program: Option<String>, instructor: Option<String>, day_of_week: Option<String> },
    ShowMyWorkouts,
    CancelWorkout { workout_id: u32 },
    BookWorkout { time: String, day: String },
}

pub struct UserRequest {
    pub user_id: u32,
    pub request_type: RequestType,
}
Отвечай только на эти запросы и возвращай ошибку если не знаешь что делать:
Запрос пользователя:
"запиши меня на тренировку в во вторник в 10:00"
