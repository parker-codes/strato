use card::Deck;

mod card;

use card::Card;

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
            let mut deck = Deck::new();
            deck.shuffle();
            self.context.deck = deck;

            // TODO: shuffle player order

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
    pub deck: Deck,
}

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Player {
    /// The player's chosen name or alias.
    pub name: &'static str,
    /// The card the user has in-hand after drawing from the deck or taking from the discard pile.
    pub holding: Option<Card>,
}

impl Player {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            holding: None,
        }
    }

    pub fn start_turn(&self, action: StartAction) -> PlayerTurnStart {
        PlayerTurnStart {
            player: self,
            action,
        }
    }

    pub fn end_turn(&self, action: EndAction) -> PlayerTurnEnd {
        PlayerTurnEnd {
            player: self,
            action,
        }
    }

    pub fn hold(&mut self, card: Card) {
        self.holding = Some(card);
    }
}

#[derive(Debug, PartialEq)]
pub struct PlayerTurnStart<'a> {
    player: &'a Player,
    action: StartAction,
}

#[derive(Debug, PartialEq)]
pub struct PlayerTurnEnd<'a> {
    player: &'a Player,
    action: EndAction,
}

/// The way the player chooses to start their turn.
#[derive(Debug, PartialEq)]
pub enum StartAction {
    DrawFromDeck,
    TakeFromDiscardPile,
}

/// The way the player chooses to end their turn.
#[derive(Debug, PartialEq)]
pub enum EndAction {
    /// Row and Column are 1-based, not 0.
    Swap { row: u8, column: u8 },
    /// Row and Column are 1-based, not 0.
    Flip { row: u8, column: u8 },
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
        assert_eq!(game.context.deck.size(), 150);
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
        let deck_snapshot = game.context.deck.clone();
        game.start();
        game.start();
        game.start();
        game.start();
        assert_eq!(game.state, GameState::Active);
        assert_eq!(deck_snapshot, game.context.deck);
    }

    #[test]
    fn can_list_players() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1);
        let player_2 = Player::new("Lexi");
        game.add_player(player_2);
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    fn cant_change_players_after_game_starts() {
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

    #[test]
    fn a_player_can_draw_and_flip() {
        let player = Player::new("Trevor");

        let start = player.start_turn(StartAction::DrawFromDeck);
        assert_eq!(
            start,
            PlayerTurnStart {
                player: &player,
                action: StartAction::DrawFromDeck
            }
        );

        let end = player.end_turn(EndAction::Flip { row: 1, column: 2 });
        assert_eq!(
            end,
            PlayerTurnEnd {
                player: &player,
                action: EndAction::Flip { row: 1, column: 2 }
            }
        );
    }

    #[test]
    fn a_player_can_take_and_swap() {
        let player = Player::new("Jon");

        let start = player.start_turn(StartAction::TakeFromDiscardPile);
        assert_eq!(
            start,
            PlayerTurnStart {
                player: &player,
                action: StartAction::TakeFromDiscardPile
            }
        );

        let end = player.end_turn(EndAction::Swap { row: 2, column: 2 });
        assert_eq!(
            end,
            PlayerTurnEnd {
                player: &player,
                action: EndAction::Swap { row: 2, column: 2 }
            }
        );
    }
}
