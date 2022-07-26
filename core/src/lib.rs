use card::{Card, Deck, DiscardPile};

mod card;

#[derive(Debug, Clone)]
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

    pub fn get_player<S: Into<String> + Clone>(&self, player_id: S) -> Option<&Player> {
        self.context
            .players
            .iter()
            .find(|p| p.id == player_id.clone().into())
    }

    pub fn start(&mut self) {
        if self.state == GameState::WaitingForPlayers && self.context.players.len() > 0 {
            self.state = GameState::Startup;

            let mut deck = Deck::new();
            deck.shuffle();
            self.context.deck = deck;

            // TODO: shuffle player order

            self.deal_cards_to_players();

            self.state = GameState::Active;
        }
    }

    fn deal_cards_to_players(&mut self) {
        if self.state == GameState::Startup {
            for player in self.context.players.iter_mut() {
                for row in 0..3 {
                    for column in 0..4 {
                        let card = self.context.deck.draw().expect("No cards left in deck.");
                        player.spread[row][column] = Some(card);
                    }
                }
            }
        }
    }

    pub fn start_player_turn<'a, S: Into<String> + Clone>(
        &mut self,
        player_id: S,
        action: StartAction,
    ) -> Result<(), String> {
        let player = self
            .context
            .players
            .iter_mut()
            .find(|p| p.id == player_id.clone().into())
            .ok_or("Couldn't find a player with that ID")?;

        match action {
            StartAction::DrawFromDeck => {
                let card = self
                    .context
                    .deck
                    .draw()
                    .ok_or("No more cards in the deck.")?;
                player.hold(card)?;
            }
            StartAction::TakeFromDiscardPile => {
                let card = self
                    .context
                    .discard_pile
                    .take()
                    .ok_or("No cards in the discard pile.")?;
                player.hold(card)?;
            }
        }

        Ok(())
    }

    pub fn end_player_turn<'a, S: Into<String> + Clone>(
        &mut self,
        player_id: S,
        action: EndAction,
    ) -> Result<(), String> {
        let player = self
            .context
            .players
            .iter_mut()
            .find(|p| p.id == player_id.clone().into())
            .ok_or("Couldn't find a player with that ID")?;

        let card_from_hand = player
            .holding
            .take()
            .ok_or("Must start turn before you can end it.")?;

        match action {
            EndAction::Swap { row, column } => {
                // TODO: validate that row and column fit within bounds
                let selected_card =
                    player.spread[row][column].ok_or("Can't swap with card that doesn't exist.")?;
                player.spread[row][column] = Some(card_from_hand);
                self.context.discard_pile.put(selected_card);
            }
            EndAction::Flip { row, column } => {
                // TODO: validate that card is not already flipped
                self.context.discard_pile.put(card_from_hand);
                // TODO: validate that row and column fit within bounds
                let mut selected_card =
                    player.spread[row][column].ok_or("Can't flip card that doesn't exist.")?;
                selected_card.flip();
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum GameState {
    #[default]
    WaitingForPlayers,
    Startup,
    Active,
    Ended,
}

#[derive(Debug, Default, Clone)]
pub struct GameContext {
    pub players: Vec<Player>,
    pub deck: Deck,
    pub discard_pile: DiscardPile,
}

type PlayerSpread = [[Option<Card>; 4]; 3];

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Player {
    /// A specified identifier.
    id: &'static str,
    /// The player's chosen name or alias.
    name: &'static str,
    /// The card the user has in-hand after drawing from the deck or taking from the discard pile.
    holding: Option<Card>,
    /// The grid of cards that each player has. Starts as 4x3 and may shrink as columns match.
    spread: PlayerSpread,
}

impl Player {
    pub fn new(id: &'static str, name: &'static str) -> Self {
        Self {
            id,
            name,
            holding: None,
            spread: [
                [None, None, None, None],
                [None, None, None, None],
                [None, None, None, None],
            ],
        }
    }

    pub fn view_spread(&self) -> PlayerSpread {
        self.spread.clone()
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
        assert_eq!(game.context.deck.size(), Deck::EMPTY_SIZE);
    }

    #[test]
    fn players_can_be_added() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("p", "Parker");
        game.add_player(player_1.clone());
        assert_eq!(game.state, GameState::WaitingForPlayers);
    }

    #[test]
    fn a_game_can_be_started() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("p", "Parker");
        game.add_player(player_1.clone());
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
    fn a_started_game_deals_cards_to_players() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("j", "Joe");
        game.add_player(player_1.clone());
        game.start();
        let joe = game.get_player(player_1.id).unwrap();
        assert_eq!(
            joe.view_spread()
                .into_iter()
                .flatten()
                .filter(|x| x.is_some() && !x.unwrap().is_visible())
                .collect::<Vec<_>>()
                .len(),
            12
        );
        assert_eq!(game.context.deck.size(), Deck::FULL_SIZE - 12);
    }

    #[test]
    fn starting_multiple_times_is_inconsequential() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("p", "Parker");
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
        let player_1 = Player::new("p", "Parker");
        game.add_player(player_1.clone());
        let player_2 = Player::new("l", "Lexi");
        game.add_player(player_2.clone());
        assert_eq!(game.list_players(), vec![player_1, player_2]);
    }

    #[test]
    fn cant_change_players_after_game_starts() {
        let mut game = StratoGame::new();
        let player_1 = Player::new("p", "Parker");
        game.add_player(player_1.clone());
        let player_2 = Player::new("l", "Lexi");
        game.add_player(player_2.clone());
        game.start();
        assert_eq!(game.state, GameState::Active);

        let player_3 = Player::new("t", "Trevor");
        game.add_player(player_3.clone());
        assert_eq!(game.list_players().len(), 2);
        assert!(game.get_player(player_3.id).is_none());
    }

    #[test]
    fn a_player_can_draw_and_flip() {
        let mut game = StratoGame::new();
        let player = Player::new("t", "Trevor");
        game.add_player(player.clone());
        game.start();

        game.start_player_turn(player.id, StartAction::DrawFromDeck)
            .expect("Couldn't start turn");
        assert!(game.get_player(player.id).unwrap().holding.is_some());
        game.end_player_turn(player.id, EndAction::Flip { row: 1, column: 2 })
            .expect("Couldn't end turn");
        assert!(game.get_player(player.id).unwrap().holding.is_none());
    }

    #[test]
    fn the_first_turn_cant_take_from_discard_pile() {
        let mut game = StratoGame::new();
        let player = Player::new("k", "Kristen");
        game.add_player(player.clone());
        game.start();

        let turn = game.start_player_turn(player.id, StartAction::TakeFromDiscardPile);
        // TODO: replace with thiserror types
        assert!(turn.is_err());
    }

    #[test]
    fn a_player_can_take_and_swap() {
        let mut game = StratoGame::new();
        let player = Player::new("j", "Jon");
        game.add_player(player.clone());
        game.start();

        // have to add a card to the discard pile first!
        game.context.discard_pile.put(Card::new(-2));

        game.start_player_turn(player.id, StartAction::TakeFromDiscardPile)
            .expect("Couldn't start turn");
        assert!(game.get_player(player.id).unwrap().holding.is_some());
        game.end_player_turn(player.id, EndAction::Swap { row: 2, column: 2 })
            .expect("Couldn't end turn");
        assert!(game.get_player(player.id).unwrap().holding.is_none());
    }
}
