use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Rights {
    rights: Vec<Rule>,
}

impl Rights {
    pub fn add_rule(&mut self, rule: Rule) {
        if self.rights.contains(&Rule::Full) {
            return;
        }
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

    pub fn get_all_rules(&self) -> Vec<(Rule, bool)> {
        Rule::list()
            .iter()
            .map(|rule| (*rule, self.has_rule(*rule)))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    Full,
    Subscription(SubscriptionsRule),
    Training(TrainingRule),
    User(UserRule),
    Settings(SettingsRule),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SettingsRule {
    ViewSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionsRule {
    ViewSubscription,
    SaleSubscription,
    // sale without subscription and restrictions
    FreeSale,
    EditSubscriptions,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum TrainingRule {
    SignupForTraining,
    CancelTrainingSignup,
    ViewSchedule,
    EditTraining,
    CancelTraining,
    Train,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum UserRule {
    ViewSelfProfile,
    EditSelfProfile,
    FindUser,
    EditUserRights,
    BlockUser,
    EditUserInfo,
}

impl Rule {
    pub fn name(&self) -> &str {
        match self {
            Rule::Full => "Full",
            Rule::Subscription(rule) => match rule {
                SubscriptionsRule::ViewSubscription => "ViewSubscription",
                SubscriptionsRule::SaleSubscription => "SaleSubscription",
                SubscriptionsRule::FreeSale => "FreeSale",
                SubscriptionsRule::EditSubscriptions => "EditSubscriptions",
            },
            Rule::Training(rule) => match rule {
                TrainingRule::SignupForTraining => "SignupForTraining",
                TrainingRule::CancelTrainingSignup => "CancelTrainingSignup",
                TrainingRule::ViewSchedule => "ViewSchedule",
                TrainingRule::EditTraining => "EditTraining",
                TrainingRule::CancelTraining => "CancelTraining",
                TrainingRule::Train => "Train",
            },
            Rule::User(rule) => match rule {
                UserRule::ViewSelfProfile => "ViewSelfProfile",
                UserRule::EditSelfProfile => "EditSelfProfile",
                UserRule::FindUser => "FindUser",
                UserRule::EditUserRights => "EditUserRights",
                UserRule::BlockUser => "BlockUser",
                UserRule::EditUserInfo => "EditUserInfo",
            },
            Rule::Settings(rule) => match rule {
                SettingsRule::ViewSettings => "ViewSettings",
            },
        }
    }

    pub fn list() -> Vec<Rule> {
        let mut rules = Vec::with_capacity(16);

        rules.push(Rule::Full);
        rules.push(Rule::Subscription(SubscriptionsRule::ViewSubscription));
        rules.push(Rule::Subscription(SubscriptionsRule::SaleSubscription));
        rules.push(Rule::Subscription(SubscriptionsRule::FreeSale));
        rules.push(Rule::Subscription(SubscriptionsRule::EditSubscriptions));
        rules.push(Rule::Training(TrainingRule::SignupForTraining));
        rules.push(Rule::Training(TrainingRule::CancelTrainingSignup));
        rules.push(Rule::Training(TrainingRule::ViewSchedule));
        rules.push(Rule::Training(TrainingRule::EditTraining));
        rules.push(Rule::Training(TrainingRule::CancelTraining));
        rules.push(Rule::Training(TrainingRule::Train));
        rules.push(Rule::User(UserRule::ViewSelfProfile));
        rules.push(Rule::User(UserRule::EditSelfProfile));
        rules.push(Rule::User(UserRule::FindUser));
        rules.push(Rule::User(UserRule::EditUserRights));
        rules.push(Rule::User(UserRule::BlockUser));
        rules.push(Rule::User(UserRule::EditUserInfo));
        rules.push(Rule::Settings(SettingsRule::ViewSettings));
        rules
    }

    pub fn id(&self) -> u32 {
        match self {
            Rule::Full => 0,
            Rule::Subscription(rule) => match rule {
                SubscriptionsRule::ViewSubscription => 1,
                SubscriptionsRule::SaleSubscription => 2,
                SubscriptionsRule::FreeSale => 3,
                SubscriptionsRule::EditSubscriptions => 4,
            },
            Rule::Training(rule) => match rule {
                TrainingRule::SignupForTraining => 5,
                TrainingRule::CancelTrainingSignup => 6,
                TrainingRule::ViewSchedule => 7,
                TrainingRule::EditTraining => 8,
                TrainingRule::CancelTraining => 9,
                TrainingRule::Train => 10,
            },
            Rule::User(rule) => match rule {
                UserRule::ViewSelfProfile => 11,
                UserRule::EditSelfProfile => 12,
                UserRule::FindUser => 13,
                UserRule::EditUserRights => 14,
                UserRule::BlockUser => 15,
                UserRule::EditUserInfo => 16,
            },
            Rule::Settings(rule) => match rule {
                SettingsRule::ViewSettings => 17,
            },
        }
    }
}

impl From<u32> for Rule {
    fn from(id: u32) -> Self {
        match id {
            0 => Rule::Full,
            1 => Rule::Subscription(SubscriptionsRule::ViewSubscription),
            2 => Rule::Subscription(SubscriptionsRule::SaleSubscription),
            3 => Rule::Subscription(SubscriptionsRule::FreeSale),
            4 => Rule::Subscription(SubscriptionsRule::EditSubscriptions),
            5 => Rule::Training(TrainingRule::SignupForTraining),
            6 => Rule::Training(TrainingRule::CancelTrainingSignup),
            7 => Rule::Training(TrainingRule::ViewSchedule),
            8 => Rule::Training(TrainingRule::EditTraining),
            9 => Rule::Training(TrainingRule::CancelTraining),
            10 => Rule::Training(TrainingRule::Train),
            11 => Rule::User(UserRule::ViewSelfProfile),
            12 => Rule::User(UserRule::EditSelfProfile),
            13 => Rule::User(UserRule::FindUser),
            14 => Rule::User(UserRule::EditUserRights),
            15 => Rule::User(UserRule::BlockUser),
            16 => Rule::User(UserRule::EditUserInfo),
            17 => Rule::Settings(SettingsRule::ViewSettings),
            _ => panic!("Invalid rule id: {}", id),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_rule_id() {
        use super::{Rule, SubscriptionsRule, TrainingRule, UserRule};
        assert_eq!(Rule::Full.id(), 0);
        assert_eq!(
            Rule::Subscription(SubscriptionsRule::ViewSubscription).id(),
            1
        );
        assert_eq!(
            Rule::Subscription(SubscriptionsRule::SaleSubscription).id(),
            2
        );
        assert_eq!(Rule::Subscription(SubscriptionsRule::FreeSale).id(), 3);
        assert_eq!(
            Rule::Subscription(SubscriptionsRule::EditSubscriptions).id(),
            4
        );
        assert_eq!(Rule::Training(TrainingRule::SignupForTraining).id(), 5);
        assert_eq!(Rule::Training(TrainingRule::CancelTrainingSignup).id(), 6);
        assert_eq!(Rule::Training(TrainingRule::ViewSchedule).id(), 7);
        assert_eq!(Rule::Training(TrainingRule::EditTraining).id(), 8);
        assert_eq!(Rule::Training(TrainingRule::CancelTraining).id(), 9);
        assert_eq!(Rule::Training(TrainingRule::Train).id(), 10);
        assert_eq!(Rule::User(UserRule::ViewSelfProfile).id(), 11);
        assert_eq!(Rule::User(UserRule::EditSelfProfile).id(), 12);
        assert_eq!(Rule::User(UserRule::FindUser).id(), 13);
        assert_eq!(Rule::User(UserRule::EditUserRights).id(), 14);
        assert_eq!(Rule::User(UserRule::BlockUser).id(), 15);
        assert_eq!(Rule::User(UserRule::EditUserInfo).id(), 16);
    }
}
