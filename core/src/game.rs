use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::card::{Deck, DiscardPile};
use crate::player::{EndAction, Player, StartAction};

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

    pub fn add_player(&mut self, player_name: &'static str) -> Result<String, String> {
        if self.state == GameState::WaitingForPlayers {
            let player_id = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect::<String>();

            let player = Player::new(player_id.clone(), player_name);
            self.context.players.push(player);

            Ok(player_id)
        } else {
            Err("Can't add players after the game has started.".into())
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
                        player
                            .spread
                            .place_at(card, row, column)
                            .expect("Can't place card");
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
            .release()
            .ok_or("Must start turn before you can end it.")?;

        match action {
            EndAction::Swap { row, column } => {
                let selected_card = player.spread.take_from(row, column)?;
                player.spread.place_at(card_from_hand, row, column)?;
                self.context.discard_pile.put(selected_card);
            }
            EndAction::Flip { row, column } => {
                player.spread.flip_at(row, column)?;
                self.context.discard_pile.put(card_from_hand);
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
