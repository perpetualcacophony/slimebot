pub trait FormatDuration {
    fn format_largest(&self) -> String;
    fn format_full(&self) -> String;
}

impl FormatDuration for chrono::Duration {
    #[rustfmt::skip]
    fn format_largest(&self) -> String {
        let (d, h, m, s) = (
            self.num_days(),
            self.num_hours(),
            self.num_minutes(),
            self.num_seconds(),
        );

        match (d, h, m, s) {
            (1  , _  , _  , _  ) => ("1 day").to_string(),
            (2.., _  , _  , _  ) => format!("{d} days"),
            (_  , 1  , _  , _  ) => ("1 hour").to_string(),
            (_  , 2.., _  , _  ) => format!("{h} hours"),
            (_  , _  , 1  , _  ) => ("1 minute").to_string(),
            (_  , _  , 2.., _  ) => format!("{m} minutes"),
            (_  , _  , _  , 1  ) => ("1 second").to_string(),
            (_  , _  , _  , 2..) => format!("{s} seconds"),
            (_  , _  , _  , _  ) => "less than a second".to_string(),
        }
    }

    fn format_full(&self) -> String {
        let mut formatted = String::new();

        if self.num_days() > 0 {
            formatted += &format!("{}d ", self.num_days());
        }

        if self.num_hours() > 0 {
            formatted += &format!("{}h ", self.num_hours() - (self.num_days() * 24));
        }

        if self.num_minutes() > 0 {
            formatted += &format!("{}m", self.num_minutes() - (self.num_hours() * 60));
        } else {
            formatted = "less than a minute".to_string();
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::FormatDuration;
    use chrono::DateTime;
    use pretty_assertions::assert_eq;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn format_full() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("2d 1h 19m", duration.format_full(),)
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn format_largest() {
        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-21T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("2 days", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-19T21:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("1 hour", duration.format_largest(),);

        let start = DateTime::parse_from_rfc3339("2024-01-19T20:00:00.000Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-01-19T20:19:00.000Z").unwrap();
        let duration = end - start;
        assert_eq!("19 minutes", duration.format_largest(),);
    }
}
