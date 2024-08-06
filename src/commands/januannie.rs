use crate::utils::{poise::ContextExt, Context};
use chrono::NaiveDate;

/// counts down to january 2025
#[poise::command(slash_command, prefix_command)]
pub async fn januannie(ctx: Context<'_>) -> crate::Result<()> {
    let result: crate::utils::poise::CommandResult = try {
        let text = text(chrono::Utc::now().date_naive());
        ctx.reply_ext(text).await?;
    };
    result?;
    Ok(())
}

const JAN_1_2025: NaiveDate =
    NaiveDate::from_ymd_opt(2025, 1, 1).expect("2025-01-01 should be a valid date");

fn text(current: NaiveDate) -> String {
    let days_until = JAN_1_2025.signed_duration_since(current).num_days();
    let days_text = if days_until == 1 {
        format!("{days_until} day")
    } else {
        format!("{days_until} days")
    };

    format!("{days_text} until Januannie!")
}

#[cfg(test)]
mod tests {
    #[test]
    fn one_day_until() {
        assert_eq!(
            super::text(
                super::JAN_1_2025
                    .checked_sub_days(chrono::Days::new(1))
                    .expect("should be in range")
            ),
            "1 day until Januannie!"
        )
    }

    #[test]
    fn one_week_until() {
        assert_eq!(
            super::text(
                super::JAN_1_2025
                    .checked_sub_days(chrono::Days::new(7))
                    .expect("should be in range"),
            ),
            "7 days until Januannie!"
        )
    }
}
