use chrono::NaiveDate;

pub fn format_data(data: &NaiveDate) -> String {
    data.format("%d.%m.%Y").to_string()
}
