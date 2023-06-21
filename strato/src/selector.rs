use crate::{game::GameContext, player::Player};

pub fn get_player<S: Into<String> + Clone>(context: GameContext, player_id: S) -> Option<Player> {
    context
        .players
        .into_iter()
        .find(|p| p.id() == player_id.clone().into())
}
