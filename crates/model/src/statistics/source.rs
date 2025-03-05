use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Source {
    Unknown {},
    Website {},
    Instagram {},
    VK {},
    YandexMap {},
    YandexDirect {},
    DirectAdds {},
    VkAdds {},
    DoubleGIS {},
    Avito {},
    Recommendation {},
    Other {},
    WebSearch {},
    OldBase {},
}

impl Source {
    pub fn iter() -> impl Iterator<Item = Source> {
        [
            Source::Unknown {},
            Source::Website {},
            Source::Instagram {},
            Source::VK {},
            Source::YandexMap {},
            Source::YandexDirect {},
            Source::DirectAdds {},
            Source::VkAdds {},
            Source::DoubleGIS {},
            Source::Avito {},
            Source::Recommendation {},
            Source::Other {},
            Source::WebSearch {},
            Source::OldBase {},
        ]
        .iter()
        .copied()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Source::Unknown {} => "Неизвестно",
            Source::DoubleGIS {} => "2ГИС",
            Source::Website {} => "Сайт",
            Source::Instagram {} => "Инстаграм",
            Source::VK {} => "ВКонтакте",
            Source::YandexMap {} => "Яндекс Карты",
            Source::DirectAdds {} => "Прямые рекламные объявления",
            Source::VkAdds {} => "Таргет ВКонтакте",
            Source::YandexDirect {} => "Яндекс Директ",
            Source::Avito {} => "Авито",
            Source::Recommendation {} => "Рекомендация",
            Source::Other {} => "Другое",
            Source::WebSearch {} => "Поиск в интернете",
            Source::OldBase {} => "Старая база",
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Source::Unknown {}
    }
}
