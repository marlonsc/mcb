//! Team registry and member manager implementations.
//!
//! Implements `TeamRegistry` and `TeamMemberManager` for managing teams and
//! team memberships.

use super::*;

sea_impl_crud!(TeamRegistry for SeaOrmEntityRepository { db: db,
    entity: team, domain: Team, label: "Team",
    create: create_team(t),
    get: get_team(id),
    list: list_teams(team::Column::OrgId => org_id),
    delete: delete_team(id)
});

#[async_trait]
impl TeamMemberManager for SeaOrmEntityRepository {
    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        sea_repo_insert!(self.db(), team_member, member, "add team member")
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        sea_repo_delete!(
            self.db(),
            team_member,
            (team_id.to_owned(), user_id.to_owned()),
            "remove team member"
        )
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        sea_repo_list!(self.db(), team_member, TeamMember, "list team members",
            team_member::Column::TeamId => team_id)
    }
}
