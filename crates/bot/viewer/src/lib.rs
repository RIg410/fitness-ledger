use teloxide::utils::markdown::escape;

pub mod day;
pub mod request;
pub mod subscription;
pub mod training;
pub mod user;
pub mod rooms;

pub fn fmt_phone(phone: Option<&str>) -> String {
    if let Some(phone) = phone {
        if phone.len() != 11 {
            return escape(phone);
        }
        let mut result = String::with_capacity(16);
        result.push_str("\\+7 \\(");
        result.push_str(&phone[1..4]);
        result.push_str("\\) ");
        result.push_str(&phone[4..7]);
        result.push_str("\\-");
        result.push_str(&phone[7..9]);
        result.push_str("\\-");
        result.push_str(&phone[9..11]);
        result
    } else {
        "Не указан".to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::fmt_phone;

    #[test]
    fn test_fmt_phone() {
        assert_eq!(
            fmt_phone(Some("71234567890")),
            "\\+7 \\(123\\) 456\\-78\\-90"
        );
    }
}
