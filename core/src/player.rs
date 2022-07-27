use crate::card::{Card, PlayerSpread};

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Player {
    // TODO: Use private key for auth?
    /// A generated identifier.
    pub id: String,
    /// The player's chosen name or alias.
    name: &'static str,
    /// The card the user has in-hand after drawing from the deck or taking from the discard pile.
    holding: Option<Card>,
    /// The grid of cards that each player has. Starts as 4x3 and may shrink as columns match.
    pub spread: PlayerSpread,
}

impl Player {
    pub fn new(id: String, name: &'static str) -> Self {
        Self {
            id,
            name,
            holding: None,
            spread: PlayerSpread::new(),
        }
    }

    /// View what the player is holding, if anything.
    pub fn holding(&self) -> Option<Card> {
        self.holding
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

    /// The Game requests the card the player is holding.
    pub fn release(&mut self) -> Option<Card> {
        self.holding.take()
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
