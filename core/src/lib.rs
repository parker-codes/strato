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

    fn draw_from_deck(&self, player: Player) -> Option<Card> {
        // TODO: implement
        Some(Card::new(-1))
    }

    fn take_from_discard_pile(&self, player: Player) -> Option<Card> {
        // TODO: implement
        Some(Card::new(2))
    }

    fn put_in_discard_pile(&self, card: Card) {
        // TODO: implement
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

type PlayerSpread = Vec<Vec<Card>>;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Player {
    /// The player's chosen name or alias.
    name: &'static str,
    /// The card the user has in-hand after drawing from the deck or taking from the discard pile.
    holding: Option<Card>,
    /// The grid of cards that each player has. Starts as 4x3 and may shrink as columns match.
    spread: PlayerSpread,
}

impl Player {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            holding: None,
            spread: vec![vec![]],
        }
    }

    pub fn view_spread(&self) -> PlayerSpread {
        self.spread.clone()
    }

    pub fn start_turn<'a>(
        &mut self,
        game: &'a StratoGame,
        action: StartAction,
    ) -> Result<(), String> {
        match action {
            StartAction::DrawFromDeck => {
                if let Some(card) = game.draw_from_deck(self.clone()) {
                    self.hold(card)?;
                }
            }
            StartAction::TakeFromDiscardPile => {
                if let Some(card) = game.take_from_discard_pile(self.clone()) {
                    self.hold(card)?;
                }
            }
        }

        Ok(())
    }

    /// The Game gives the player the card they drew or took during the start of their
    /// turn, to use when they end their turn.
    pub fn hold(&mut self, mut card: Card) -> Result<(), String> {
        if let Some(card_in_hand) = self.holding {
            return Err(format!("Already holding {card_in_hand:#?}"));
        }

        card.flip();
        self.holding = Some(card);

        Ok(())
    }

    pub fn end_turn<'a>(&mut self, game: &'a StratoGame, action: EndAction) -> Result<(), String> {
        let card_from_hand = self
            .holding
            .take()
            .ok_or("Must start turn before you can end it.")?;

        match action {
            EndAction::Swap { row, column } => {
                // TODO: validate that row and column fit within bounds
                // let selected_card = self.spread[row][column];

                // self.spread[row][column] = card_from_hand;

                // game.put_in_discard_pile(selected_card);
            }
            EndAction::Flip { row, column } => {
                game.put_in_discard_pile(card_from_hand);

                // TODO: validate that row and column fit within bounds
                // let mut selected_card = self.spread[row][column];
                // selected_card.flip();
            }
        }

        Ok(())
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
    /// Row and Column are 0-based.
    Swap { row: usize, column: usize },
    /// Row and Column are 0-based.
    Flip { row: usize, column: usize },
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
        game.add_player(player_1.clone());
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn a_game_can_be_started() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1.clone());
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
        game.add_player(player_1.clone());
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
        game.add_player(player_1.clone());
        let player_2 = Player::new("Lexi");
        game.add_player(player_2.clone());
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    fn cant_change_players_after_game_starts() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("Parker");
        game.add_player(player_1.clone());
        let player_2 = Player::new("Lexi");
        game.add_player(player_2.clone());
        game.start();
        assert_eq!(game.state, GameState::Active);

        let player_3 = Player::new("Trevor");
        game.add_player(player_3.clone());
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    fn a_player_can_draw_and_flip() {
        let mut game = StratoGame::new();
        let mut player = Player::new("Trevor");
        game.add_player(player.clone());
        game.start();

        player
            .start_turn(&game, StartAction::DrawFromDeck)
            .expect("Couldn't start turn");
        assert!(&player.holding.is_some());
        player
            .end_turn(&game, EndAction::Flip { row: 1, column: 2 })
            .expect("Couldn't end turn");
        assert!(&player.holding.is_none());
    }

    #[test]
    fn a_player_can_take_and_swap() {
        let mut game = StratoGame::new();
        let mut player = Player::new("Jon");
        game.add_player(player.clone());
        game.start();

        player
            .start_turn(&game, StartAction::TakeFromDiscardPile)
            .expect("Couldn't start turn");
        assert!(&player.holding.is_some());
        player
            .end_turn(&game, EndAction::Swap { row: 2, column: 2 })
            .expect("Couldn't end turn");
        assert!(&player.holding.is_none());
    }

    // #[test]
    // fn a_game_turn_can_be_played_by_player() {
    //     let mut game = StratoGame::new();
    //     let mut player = Player::new("Parker");
    //     game.add_player(player.clone());
    //     game.start();

    //     player.start_turn(&game, StartAction::DrawFromDeck);
    // }

    fn card_is_held_after_starting_turn() {}

    fn card_is_not_held_after_ending_turn() {}
}
