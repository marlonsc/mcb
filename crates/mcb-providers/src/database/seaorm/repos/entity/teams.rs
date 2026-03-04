//! Team registry and member manager implementations.
//!
//! Implements `TeamRegistry` and `TeamMemberManager` for managing teams and
//! team memberships.

use super::*;

#[async_trait]
impl TeamRegistry for SeaOrmEntityRepository {
    async fn create_team(&self, t: &Team) -> Result<()> {
        sea_repo_insert!(self.db(), team, t, "create team")
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        sea_repo_get!(self.db(), team, Team, "Team", id, "get team")
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        sea_repo_list!(self.db(), team, Team, "list teams", team::Column::OrgId => org_id)
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        sea_repo_delete!(self.db(), team, id, "delete team")
    }
}

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
        sea_repo_list!(self.db(), team_member, TeamMember, "list team members", team_member::Column::TeamId => team_id)
    }
}
