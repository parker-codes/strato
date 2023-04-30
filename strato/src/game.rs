use std::rc::Rc;

use anyhow::Result;
use rand::distributions::Alphanumeric;
use rand::Rng;
use thiserror::Error;

use crate::card::{Deck, DiscardPile};
use crate::player::{EndAction, Player, StartAction};

#[derive(Error, Debug, PartialEq)]
pub enum GameStartupError {
    #[error("The game has already been started.")]
    GameAlreadyStarted,
    #[error("Can't add players after the game has started.")]
    PlayersListLocked,
    #[error("Not enough players to start the game.")]
    NotEnoughPlayers,
    #[error(transparent)]
    PlayerSpreadError(#[from] crate::card::SpreadActionError),
    #[error("No more cards in the deck.")]
    DeckEmpty,
}

#[derive(Error, Debug, PartialEq)]
pub enum PlayerTurnError {
    #[error("Couldn't find a player with that ID.")]
    PlayerDoesntExist,
    #[error("The first player has already been determined.")]
    NotDeterminingFirstPlayer,
    #[error("You have already flipped your cards to determine who goes first.")]
    TooManyCardsFlipped,
    #[error("The game has not started yet.")]
    GameNotStarted,
    #[error("You can't start your turn when it is already started.")]
    TurnAlreadyStarted,
    #[error("You must start turn before you can end it.")]
    TurnNotStarted,
    #[error("It is not your turn.")]
    NotYourTurn,
    #[error(transparent)]
    PlayerActionError(#[from] crate::player::PlayerActionError),
    #[error(transparent)]
    PlayerSpreadError(#[from] crate::card::SpreadActionError),
    #[error("No more cards in the deck.")]
    DeckEmpty,
    #[error("No more cards in the discard pile.")]
    DiscardPileEmpty,
}

#[derive(Debug, Clone)]
pub struct StratoGame<'s> {
    pub state: GameState,
    pub context: GameContext,
    subscriber: Option<Rc<Subscriber<'s>>>,
}

impl<'s> StratoGame<'s> {
    pub fn new() -> Self {
        Self {
            state: GameState::default(),
            context: GameContext::default(),
            subscriber: None,
        }
    }

    fn update_state(&mut self, state: GameState) {
        self.state = state;
        self.notify(GameEvent::StateChange(&self.state));
    }

    pub fn subscribe(&mut self, f: impl Fn(GameEvent) + 's) {
        self.subscriber = Some(Rc::new(Subscriber::new(f)));
    }

    pub fn unsubscribe(&mut self) {
        self.subscriber = None;
    }

    fn notify(&self, event: GameEvent) {
        if let Some(subscriber) = &self.subscriber {
            (subscriber.0)(event);
        }
    }

    pub fn add_player(&mut self, player_name: &'static str) -> Result<String, GameStartupError> {
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
            Err(GameStartupError::PlayersListLocked)
        }
    }

    pub fn list_players(&self) -> Vec<Player> {
        self.context.players.clone()
    }

    pub fn get_player<S: Into<String> + Clone>(&self, player_id: S) -> Option<&Player> {
        self.context
            .players
            .iter()
            .find(|p| p.id() == player_id.clone().into())
    }

    pub fn start(&mut self) -> Result<(), GameStartupError> {
        self.handle_start(GameOptions::default())
    }

    pub fn start_with_options(&mut self, options: GameOptions) -> Result<(), GameStartupError> {
        self.handle_start(options)
    }

    fn handle_start(&mut self, options: GameOptions) -> Result<(), GameStartupError> {
        if self.state == GameState::Active {
            return Err(GameStartupError::GameAlreadyStarted);
        } else if self.context.players.len() < 2 {
            return Err(GameStartupError::NotEnoughPlayers);
        } else if self.state == GameState::WaitingForPlayers {
            self.update_state(GameState::Startup);

            self.context.deck.shuffle();
            let top_card = self.context.deck.draw().unwrap();
            self.context.discard_pile.put(top_card);
            // TODO: shuffle player order?
            self.deal_cards_to_players()?;

            if let Some(first_player_idx) = options.first_player_idx {
                self.context.current_player_idx = Some(first_player_idx);
                self.update_state(GameState::Active);
            } else {
                self.update_state(GameState::DetermineFirstPlayer);
            }
        }

        Ok(())
    }

    fn deal_cards_to_players(&mut self) -> Result<(), GameStartupError> {
        if self.state == GameState::Startup {
            for player in self.context.players.iter_mut() {
                for row in 0..3 {
                    for column in 0..4 {
                        let card = self
                            .context
                            .deck
                            .draw()
                            .ok_or(GameStartupError::DeckEmpty)?;
                        player.spread.place_at(card, row, column)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn player_flip_to_determine_who_is_first<'a, S: Into<String> + Clone>(
        &mut self,
        player_id: S,
        row: usize,
        column: usize,
    ) -> Result<(), PlayerTurnError> {
        if self.state != GameState::DetermineFirstPlayer {
            return Err(PlayerTurnError::NotDeterminingFirstPlayer);
        }

        let player = self
            .context
            .players
            .iter_mut()
            .find(|p| p.id() == player_id.clone().into())
            .ok_or(PlayerTurnError::PlayerDoesntExist)?;

        if player.spread.flipped_cards() >= 2 {
            return Err(PlayerTurnError::TooManyCardsFlipped);
        }

        player.spread.flip_at(row, column)?;

        if let Some(first_player_idx) = self.check_if_first_player_determined() {
            self.context.current_player_idx = Some(first_player_idx);
            self.update_state(GameState::Active);
        }

        Ok(())
    }

    fn check_if_first_player_determined(&self) -> Option<usize> {
        let all_players_have_two_cards_flipped = self
            .context
            .players
            .iter()
            .all(|p| p.spread.flipped_cards() == 2);

        if all_players_have_two_cards_flipped {
            let highest_score_idx = self
                .context
                .players
                .iter()
                .enumerate()
                .max_by_key(|(_, p)| p.spread.score())
                .map(|(idx, _)| idx)
                .unwrap();
            return Some(highest_score_idx);
        }

        None
    }

    pub fn start_player_turn<'a, S: Into<String> + Clone>(
        &mut self,
        player_id: S,
        action: StartAction,
    ) -> Result<(), PlayerTurnError> {
        if self.state != GameState::Active {
            return Err(PlayerTurnError::GameNotStarted);
        }

        let player_idx = self
            .context
            .players
            .iter()
            .position(|p| p.id() == player_id.clone().into())
            .ok_or(PlayerTurnError::PlayerDoesntExist)?;

        self.check_if_player_turn(player_idx)?;

        let player = &mut self.context.players[player_idx];

        if player.holding().is_some() {
            return Err(PlayerTurnError::TurnAlreadyStarted);
        }

        match action {
            StartAction::DrawFromDeck => {
                let card = self.context.deck.draw().ok_or(PlayerTurnError::DeckEmpty)?;
                player.hold(card)?;
            }
            StartAction::TakeFromDiscardPile => {
                let card = self
                    .context
                    .discard_pile
                    .take()
                    .ok_or(PlayerTurnError::DiscardPileEmpty)?;
                player.hold(card)?;
            }
        }

        Ok(())
    }

    pub fn end_player_turn<'a, S: Into<String> + Clone>(
        &mut self,
        player_id: S,
        action: EndAction,
    ) -> Result<(), PlayerTurnError> {
        if self.state != GameState::Active {
            return Err(PlayerTurnError::GameNotStarted);
        }

        let player_idx = self
            .context
            .players
            .iter()
            .position(|p| p.id() == player_id.clone().into())
            .ok_or(PlayerTurnError::PlayerDoesntExist)?;

        self.check_if_player_turn(player_idx)?;

        let player = &mut self.context.players[player_idx];

        let card_from_hand = player.release().ok_or(PlayerTurnError::TurnNotStarted)?;

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

        match action {
            EndAction::Swap { column, .. } | EndAction::Flip { column, .. } => {
                player.spread.remove_column_if_matches(column)?;
            }
        }

        // TODO: if all current player's cards are flipped, begin final round

        // TODO: if all cards in game are flipped, end game

        self.advance_player_turn();

        Ok(())
    }

    fn advance_player_turn(&mut self) {
        if let Some(current_player_idx) = self.context.current_player_idx {
            self.context.current_player_idx =
                Some((current_player_idx + 1) % self.context.players.len());
        }
    }

    fn check_if_player_turn(&self, player_idx: usize) -> Result<(), PlayerTurnError> {
        if let Some(current_player_idx) = self.context.current_player_idx {
            if player_idx != current_player_idx {
                return Err(PlayerTurnError::NotYourTurn);
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
    DetermineFirstPlayer,
    Active,
    LastRound, // TODO: everyone gets one last go
    Ended,
}

#[derive(Debug, Default, Clone)]
pub struct GameContext {
    pub players: Vec<Player>,
    pub current_player_idx: Option<usize>,
    pub deck: Deck,
    pub discard_pile: DiscardPile,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent<'a> {
    StateChange(&'a GameState),
}

struct Subscriber<'s>(Box<dyn Fn(GameEvent) + 's>);

impl std::fmt::Debug for Subscriber<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscriber")
    }
}

impl<'s> Subscriber<'s> {
    fn new<F: Fn(GameEvent) + 's>(f: F) -> Self {
        Self(Box::new(f))
    }
}

#[derive(Default, Debug)]
pub struct GameOptions {
    pub first_player_idx: Option<usize>,
}
