pub struct StratoMachine {
    pub state: GameState,
    pub context: GameContext,
}

impl StratoMachine {
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
            context: GameContext::default(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.context.players.push(player);
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
    players: Vec<Player>,
}

#[derive(Debug, Default)]
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
        let game = StratoMachine::new();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn players_can_be_added() {
        let mut game = StratoMachine::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn a_game_can_be_started() {
        let mut game = StratoMachine::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        assert_eq!(game.state, GameState::Active);
    }

    #[test]
    fn cant_start_without_players() {
        let mut game = StratoMachine::new();
        game.start();
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn starting_multiple_times_is_inconsequential() {
        let mut game = StratoMachine::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        game.start();
        game.start();
        game.start();
        game.start();
        game.start();
        assert_eq!(game.state, GameState::Active);
    }
}
