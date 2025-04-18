use anchor_lang::prelude::*;

/// The main struct that stores all tip jar data on-chain
/// This is created as PDA owned by the program
#[account]
pub struct TipJar {
    /// whether tips can currently be sent to this jar
    pub is_active: bool,
    /// whether this jar is private (only owner can send tips)
    pub is_private: bool,
    /// the wallet that owns this tip jar and can withdraw funds
    pub owner: Pubkey,
    /// description of what this tip jar is for
    pub description: String,
    /// category tag for the tip jar (e.g., "content creation", "community", etc.)
    pub category: String,
    /// fundraising goal amount in lamports (1 SOL = 1,000,000,000 lamports)
    pub goal: u64,
    /// total amount of SOL received in lamports
    pub total_received: u64,
    /// history of recent tips, implemented as a circular buffer
    pub tips_history: Vec<Tip>,
    /// current position in the circular buffer
    pub last_tip_index: u16,
    /// total count pf all tips ever received (not limited by the buffer size)
    pub total_tips_count: u32,
    /// PDA bump used to derive this account's address
    pub bump: u8,
}

/// Implementation for tipjar with space calculation and constants
impl TipJar {
   // Base account discriminator - Anchor uses this to identify account types
    const DISCRIMINATOR_LENGTH: usize = 8;

   //static fields total size
    const STATIC_SIZE: usize = 
   1 + // is_active
   1 + // is_private
   32 + // owner (Pubkey)
   8 + // goal
   8 + // total_received
   1 + // bump
   2 + // last_tip_index
   4; // total_tips_count

    // dynamic fields calculation
    const MAX_DESCRIPTION_LEN: usize = 200;
    const MAX_CATEGORY_LEN: usize = 100;
    // Maximum number of tips to store in history
    pub const MAX_TIPS_HISTORY_LEN: usize = 100; // reduced for efficient space usage

    /// Calculates the total space needed for this account
    pub fn space() -> usize {
        Self::DISCRIMINATOR_SIZE + // account discriminator
        Self::STATIC_SIZE + // static fields
        4 + Self::MAX_DESCRIPTION_LEN + // String prefix(4) + max chars description
        4 + Self::MAX_CATEGORY_LEN + // String prefix(4) + max chars category
        4 + (Self::MAX_TIP_HISTORY_LEN * Tip::SIZE) // Vec prefix(4) + entries
    }

    // total length constant used in account initialization
    pub const LEN: usize = Self::space();
}

// Represents a single tip with sender, amount and message
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Tip {
   /// public key of the tip sender
    pub sender: Pubkey,
   /// amount of SOL sent in lamports
    pub amount: u64,
   /// whether this tip is publicly visible or anonymous
    pub visibility: Visibility,
   /// optional message included with the tip
    pub memo: String,
   ///  unix timestamps when the tip was sent
    pub timestamp: u64,
}

// Implementation for tip with space calculation
impl Tip {
   /// size of a single tip in bytes
    pub const SIZE: usize = 32 + // sender (Pubkey)
    8 + // amount
    1 + // visibility (enum)
    (4 + 100) + // memo length (u32)
    8; // timestamp (u64) 
}

/// Enum for tip visibility
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum Visibility {
    /// Tip is publicly visible with sender info
    Public,
    /// Tip is anonymous and only amount is visible
    Anonymous,
}