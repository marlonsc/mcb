//! Team registry and member manager implementations.
//!
//! Implements `TeamRegistry` and `TeamMemberManager` for managing teams and
//! team memberships.

use super::*;

#[async_trait]
impl TeamRegistry for SeaOrmEntityRepository {
    async fn create_team(&self, t: &Team) -> Result<()> {
        sea_insert!(self, team, t)
    }

    async fn get_team(&self, id: &str) -> Result<Team> {
        sea_get!(self, team, Team, "Team", id)
    }

    async fn list_teams(&self, org_id: &str) -> Result<Vec<Team>> {
        sea_list!(self, team, Team, team::Column::OrgId => org_id)
    }

    async fn delete_team(&self, id: &str) -> Result<()> {
        sea_delete!(self, team, id)
    }
}

#[async_trait]
impl TeamMemberManager for SeaOrmEntityRepository {
    async fn add_team_member(&self, member: &TeamMember) -> Result<()> {
        sea_insert!(self, team_member, member)
    }

    async fn remove_team_member(&self, team_id: &str, user_id: &str) -> Result<()> {
        sea_delete!(self, team_member, (team_id.to_owned(), user_id.to_owned()))
    }

    async fn list_team_members(&self, team_id: &str) -> Result<Vec<TeamMember>> {
        sea_list!(self, team_member, TeamMember, team_member::Column::TeamId => team_id)
    }
}
