use serde::{Deserialize, Serialize};
use strum::{EnumIter, FromRepr, IntoEnumIterator as _};

const CUSTOMER_RULES: [Rule; 1] = [Rule::ViewProfile];

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Rights {
    full: bool,
    rights: Vec<Rule>,
}

impl Rights {
    pub fn full() -> Self {
        Rights {
            full: true,
            rights: vec![],
        }
    }

    pub fn customer() -> Self {
        Rights {
            full: false,
            rights: CUSTOMER_RULES.to_vec(),
        }
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn add_rule(&mut self, rule: Rule) {
        if self.full {
            return;
        }
        self.rights.push(rule);
    }

    pub fn remove_rule(&mut self, rule: Rule) {
        if self.full {
            return;
        }
        self.rights.retain(|r| r != &rule);
    }

    pub fn has_rule(&self, rule: Rule) -> bool {
        if self.full {
            return true;
        }
        self.rights.contains(&rule)
    }

    pub fn ensure(&self, rule: Rule) -> eyre::Result<()> {
        if !self.has_rule(rule) {
            return Err(eyre::eyre!("User has no rights to perform this action"));
        }
        Ok(())
    }

    pub fn get_all_rules(&self) -> Vec<(Rule, bool)> {
        Rule::list()
            .iter()
            .map(|rule| (*rule, self.has_rule(*rule)))
            .collect()
    }
}

#[derive(FromRepr, EnumIter, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Rule {
    ViewProfile,

    // User menu
    ViewUsers,
    EditUserRights,
    BlockUser,
    EditUserInfo,

    // Training
    Train,
    EditTraining,
    CancelTraining,
    CreateTraining,
    EditSchedule,
}

impl Rule {
    pub fn name(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }

    pub fn list() -> Vec<Rule> {
        Rule::iter().collect()
    }

    pub fn id(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for Rule {
    type Error = eyre::Error;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Rule::from_repr(value).ok_or_else(|| eyre::eyre!("Invalid rule: {}", value))
    }
}