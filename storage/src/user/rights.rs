use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Rights {
    rights: Vec<Rule>,
}

impl Rights {
    pub fn add_rule(&mut self, rule: Rule) {
        self.rights.push(rule);
    }

    pub fn remove_rule(&mut self, rule: Rule) {
        self.rights.retain(|r| r != &rule);
    }

    pub fn has_rule(&self, rule: Rule) -> bool {
        if self.rights.contains(&Rule::Full) {
            return true;
        }
        self.rights.contains(&rule)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Rule {
    Full,
    Subscription(SubscriptionsRule),
    Training(TrainingRule),
    User(UserRule),
    Settings(SettingsRule),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SettingsRule {
    ViewSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum SubscriptionsRule {
    ViewSubscription,
    SaleSubscription,
    // sale without subscription and restrictions
    FreeSale,
    EditSubscriptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TrainingRule {
    SignupForTraining,
    CancelTrainingSignup,
    ViewSchedule,
    EditTraining,
    CancelTraining,
    Train
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserRule {
    ViewSelfProfile,
    EditSelfProfile,
    FindUser,
    EditUser,
}
