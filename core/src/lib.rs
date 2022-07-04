pub struct StratoGame {
    pub state: GameState,
    pub context: GameContext,
}

impl StratoGame {
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
            context: GameContext::default(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        if self.state == GameState::WaitingForPlayers {
            self.context.players.push(player);
        }
    }

    pub fn list_players(&self) -> Vec<Player> {
        self.context.players.clone()
    }

    pub fn start(&mut self) {
        if self.state == GameState::WaitingForPlayers && self.context.players.len() > 0 {
            self.state = GameState::Active;
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum GameState {
    #[default]
    WaitingForPlayers,
    Active,
    Ended,
}

#[derive(Debug, Default)]
pub struct GameContext {
    pub players: Vec<Player>,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Player {
    pub name: &'static str,
}

impl Player {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_game_can_be_initialized() {
        let game = StratoGame::new();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn players_can_be_added() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn a_game_can_be_started() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        assert_eq!(game.state, GameState::Active);
    }

    #[test]
    fn cant_start_without_players() {
        let mut game = StratoGame::new();
        game.start();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn starting_multiple_times_is_inconsequential() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        game.start();
        game.start();
        game.start();
        game.start();
        assert_eq!(game.state, GameState::Active);
    }

    #[test]
    pub fn can_list_players() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        let player_2 = Player::new("Lexi");
        game.add_player(player_2);
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    pub fn cant_change_players_after_game_starts() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        let player_2 = Player::new("Lexi");
        game.add_player(player_2);
        game.start();
        assert_eq!(game.state, GameState::Active);

        let player_3 = Player::new("Trevor");
        game.add_player(player_3);
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }
}
