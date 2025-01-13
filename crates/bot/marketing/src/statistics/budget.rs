use bot_core::context::Context;
use eyre::Error;

use super::Range;

pub async fn send_statistic(ctx: &mut Context, range: Range) -> Result<(), Error> {
    let group_by = range.group_by();
    let (from, to) = range.range()?;
    Ok(())
}