use std::{collections::HashMap, sync::Arc};

use arc_swap::ArcSwap;
use chrono::{DateTime, Datelike as _, Local, NaiveDate};
use model::statistics::month::MonthStatistics;

pub struct CacheEntry {
    last_update: Option<DateTime<Local>>,
    pub value: HashMap<NaiveDate, MonthStatistics>,
}

pub struct StatCache {
    inner: ArcSwap<CacheEntry>,
}

impl StatCache {
    pub fn new() -> Self {
        Self {
            inner: ArcSwap::new(Arc::new(CacheEntry {
                last_update: None,
                value: Default::default(),
            })),
        }
    }

    pub fn set_value(&self, value: HashMap<NaiveDate, MonthStatistics>) {
        let value = CacheEntry {
            last_update: Some(Local::now()),
            value,
        };
        self.inner.store(Arc::new(value));
    }

    pub fn get_value(&self) -> Option<Arc<CacheEntry>> {
        let now = Local::now();
        let day = now.day();
        let month = now.month();

        let entry = self.inner.load();
        if let Some(last_update) = entry.last_update {
            if last_update.day() == day && last_update.month() == month {
                return Some(entry.clone());
            }
        }
        None
    }
}
