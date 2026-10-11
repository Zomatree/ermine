use freya::prelude::*;
use itertools::Itertools;
use stoat_models::v0;

use crate::color::parse_fill;

pub fn member_display_color(member: &v0::Member, server: &v0::Server) -> Option<Fill> {
    let mut roles = member
        .roles
        .iter()
        .filter_map(|id| server.roles.get(id))
        .collect::<Vec<_>>();

    roles.sort_by(|a, b| a.rank.cmp(&b.rank));

    let color = roles
        .into_iter()
        .filter_map(|role| role.colour.as_ref())
        .next()?;

    parse_fill(color)
}

pub fn member_role_icon(member: &v0::Member, server: &v0::Server) -> Option<(v0::File, String)> {
    let mut roles = member
        .roles
        .iter()
        .filter_map(|id| server.roles.get(id))
        .collect::<Vec<_>>();

    roles.sort_by(|a, b| a.rank.cmp(&b.rank));

    roles
        .into_iter()
        .filter_map(|role| role.icon.clone().map(|icon| (icon, role.name.clone())))
        .next()
}

pub fn display_name<'a>(
    user: &'a v0::User,
    member: Option<&'a v0::Member>,
    message: Option<&'a v0::Message>,
) -> &'a str {
    message
        .and_then(|msg| {
            msg.masquerade
                .as_ref()
                .and_then(|masq| masq.name.as_deref())
        })
        .or_else(|| member.and_then(|member| member.nickname.as_deref()))
        .or(user.display_name.as_deref())
        .unwrap_or(&user.username)
}

pub fn get_member_rank(member: &v0::Member, server: &v0::Server) -> i64 {
    if &member.id.user == &server.owner {
        i64::MIN
    } else {
        member
            .roles
            .iter()
            .filter_map(|id| server.roles.get(id))
            .sorted_by(|a, b| a.rank.cmp(&b.rank))
            .map(|r| r.rank)
            .next()
            .unwrap_or(i64::MAX)
    }
}

pub fn is_inferior(member: &v0::Member, target: &v0::Member, server: &v0::Server) -> bool {
    get_member_rank(member, server) < get_member_rank(target, server)
}
