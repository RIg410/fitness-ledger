use bot_core::{context::Context, widget::Jmp, CommonLocation};
use bot_marketing::requests::Requests;
use bot_users::profile::UserProfile;
use eyre::Result;
use model::rights::Rule;

pub async fn handle_common_location(ctx: &mut Context, location: CommonLocation) -> Result<Jmp> {
    Ok(match location {
        CommonLocation::Profile(object_id) => {
            if ctx.has_right(Rule::ViewUsers) {
                UserProfile::new(object_id).into()
            } else {
                Jmp::Stay
            }
        }
        CommonLocation::Request(object_id) => {
            if ctx.has_right(Rule::ViewMarketingInfo) {
                Requests::new(None, true, Some(object_id)).into()
            } else {
                Jmp::Stay
            }
        }
    })
}
