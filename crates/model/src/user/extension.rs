use bson::oid::ObjectId;
use chrono::{DateTime, Datelike as _, Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display, Formatter};

use super::comments::Comment;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserExtension {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub birthday: Option<Birthday>,
    #[serde(default)]
    pub notification_mask: NotificationMask,
    pub ai_message_prompt: Option<String>,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Birthday {
    day: u32,
    month: u32,
    year: i32,
}

impl Birthday {
    pub fn new(dt: DateTime<Local>) -> Birthday {
        Birthday {
            day: dt.day(),
            month: dt.month(),
            year: dt.year(),
        }
    }
}

impl Display for Birthday {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}.{:02}.{}", self.day, self.month, self.year)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NotificationMask {
    mask: u32,
}

impl Debug for NotificationMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for i in 0..24 {
            if self.get_hour(i) {
                write!(f, "{:02}:00 ", i)?;
            }
        }
        Ok(())
    }
}

impl NotificationMask {
    pub fn hours(&self) -> [bool; 24] {
        let mut hours = [false; 24];
        for i in 0..24 {
            hours[i] = self.mask & (1 << i) != 0;
        }
        hours
    }

    pub fn set_hours(&mut self, hours: [bool; 24]) {
        self.mask = 0;
        for i in 0..24 {
            if hours[i] {
                self.mask |= 1 << i;
            }
        }
    }

    pub fn set_hour(&mut self, hour: u32, value: bool) {
        if value {
            self.mask |= 1 << hour;
        } else {
            self.mask &= !(1 << hour);
        }
    }

    pub fn get_hour(&self, hour: u32) -> bool {
        self.mask & (1 << hour) != 0
    }

    pub fn is_disabled(&self) -> bool {
        self.mask == 0
    }

    pub fn to_nearest_time(&self, time: DateTime<Local>) -> DateTime<Local> {
        let mut time = time;
        for _ in 0..24 {
            if self.get_hour(time.hour()) {
                return time;
            }
            time = (time + chrono::Duration::hours(1)).with_minute(0).unwrap();
        }
        time
    }
}

impl Default for NotificationMask {
    fn default() -> Self {
        NotificationMask {
            mask: 0b00000000_0111_1111_1111_1111_0000_0000,
        }
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use chrono::TimeZone as _;

    use super::*;

    #[test]
    fn test_to_nearest_time() {
        let mut mask = NotificationMask::default();
        mask.set_hour(9, true);
        mask.set_hour(10, true);
        mask.set_hour(11, true);
        mask.set_hour(12, true);
        mask.set_hour(13, true);
        mask.set_hour(14, true);
        mask.set_hour(15, true);
        mask.set_hour(16, true);
        mask.set_hour(17, true);
        mask.set_hour(18, true);
        mask.set_hour(19, true);
        mask.set_hour(20, true);

        let nearest_time = mask.to_nearest_time(Local.ymd(2021, 1, 1).and_hms(0, 30, 0));
        assert_eq!(nearest_time, Local.ymd(2021, 1, 1).and_hms(8, 0, 0));

        let nearest_time = mask.to_nearest_time(Local.ymd(2021, 1, 1).and_hms(9, 30, 0));
        assert_eq!(nearest_time, Local.ymd(2021, 1, 1).and_hms(9, 30, 0));

        let nearest_time = mask.to_nearest_time(Local.ymd(2021, 1, 1).and_hms(20, 30, 0));
        assert_eq!(nearest_time, Local.ymd(2021, 1, 1).and_hms(20, 30, 0));

        let nearest_time = mask.to_nearest_time(Local.ymd(2021, 1, 1).and_hms(23, 30, 0));
        assert_eq!(nearest_time, Local.ymd(2021, 1, 2).and_hms(8, 0, 0));
    }

    #[test]
    fn test_notification_mask() {
        let mut mask = NotificationMask { mask: 0 };
        mask.set_hour(0, true);
        mask.set_hour(1, true);
        mask.set_hour(2, true);
        mask.set_hour(3, true);
        mask.set_hour(4, true);
        mask.set_hour(5, true);
        mask.set_hour(6, true);
        mask.set_hour(7, true);
        mask.set_hour(8, true);
        mask.set_hour(9, true);
        mask.set_hour(10, true);
        mask.set_hour(11, true);
        mask.set_hour(12, true);
        mask.set_hour(13, true);
        mask.set_hour(14, true);
        mask.set_hour(15, true);
        mask.set_hour(16, true);
        mask.set_hour(17, true);
        mask.set_hour(18, true);
        mask.set_hour(19, true);
        mask.set_hour(20, true);
        mask.set_hour(21, true);
        mask.set_hour(22, true);
        mask.set_hour(23, true);

        assert_eq!(mask.hours(), [true; 24]);

        mask.set_hour(0, false);
        mask.set_hour(1, false);
        mask.set_hour(2, false);
        mask.set_hour(3, false);
        mask.set_hour(4, false);
        mask.set_hour(5, false);
        mask.set_hour(6, false);
        mask.set_hour(7, false);
        mask.set_hour(8, false);
        mask.set_hour(9, false);
        mask.set_hour(10, false);
        mask.set_hour(11, false);
        mask.set_hour(12, false);
        mask.set_hour(13, false);
        mask.set_hour(14, false);
        mask.set_hour(15, false);
        mask.set_hour(16, false);
        mask.set_hour(17, false);
        mask.set_hour(18, false);
        mask.set_hour(19, false);
        mask.set_hour(20, false);
        mask.set_hour(21, false);
        mask.set_hour(22, false);
        mask.set_hour(23, false);

        assert_eq!(mask.hours(), [false; 24]);
    }

    #[test]
    fn test_random_notification_mask() {
        let mut mask = NotificationMask { mask: 0 };
        mask.set_hour(0, true);
        mask.set_hour(1, false);
        mask.set_hour(2, true);
        mask.set_hour(3, false);
        mask.set_hour(4, true);
        mask.set_hour(5, false);
        mask.set_hour(6, true);
        mask.set_hour(7, false);
        mask.set_hour(8, true);
        mask.set_hour(9, false);
        mask.set_hour(10, true);
        mask.set_hour(11, false);
        mask.set_hour(12, true);
        mask.set_hour(13, false);
        mask.set_hour(14, true);
        mask.set_hour(15, false);
        mask.set_hour(16, true);
        mask.set_hour(17, false);
        mask.set_hour(18, true);
        mask.set_hour(19, false);
        mask.set_hour(20, true);
        mask.set_hour(21, false);
        mask.set_hour(22, true);
        mask.set_hour(23, false);

        assert_eq!(
            mask.hours(),
            [
                true, false, true, false, true, false, true, false, true, false, true, false, true,
                false, true, false, true, false, true, false, true, false, true, false
            ]
        );
    }
}
