use std::collections::HashMap;

pub struct TrainingsStat {
    pub trainings: HashMap<String, TrainingStat>,
    pub instructors: HashMap<String, TrainingStat>,
}

pub struct TrainingStat {
    pub trainings_count: u64,
    pub total_clients: u64,
    pub total_earned: i64,
    pub trainings_with_out_clients: u64,
    pub canceled_trainings: u64,
}
