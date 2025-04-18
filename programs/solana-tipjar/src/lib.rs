use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use crate::state::*;

declare_id!("6U7ezSr7phBBojC5PRUuutUNFpMiDxUxXeiKfjTZduMs");

// Import state module to access TipJar and Tip structs
pub mod state;

#[program]
pub mod tipjar {
    use super::*;

    /// Creates a new tip jar with the provided details
    /// Takes description, category, and goal amount
    pub fn initialize_tipjar(ctx: Context<InitializeTipJar>, description: String, category: String, goal: u64) -> Result<()> {
        // Validate input parameters
        require!(goal > 0, TipJarError::InvalidGoal);
        require!(description.len() <= 200, TipJarError::DescriptionTooLong);
        require!(category.len() <= 100, TipJarError::CategoryTooLong);
        
        let tip_jar = &mut ctx.accounts.tipjar;
        let user = &ctx.accounts.user;
        
        // Initialize TipJar fields
        tip_jar.description = description;
        tip_jar.category = category;
        tip_jar.goal = goal;
        tip_jar.total_received = 0;
        tip_jar.is_active = true;
        tip_jar.owner = user.key();
        tip_jar.bump = *ctx.bumps.get("tipjar").unwrap();
        
        Ok(())
    }

    /// Sends a tip to a tip jar with optional message and visibility setting
    pub fn send_tip(ctx: Context<SendTip>, amount: u64, visibility: Visibility, memo: String) -> Result<()> {
        // Validate inputs
        require!(amount > 0, TipJarError::InvalidAmount);
        require!(memo.len() <= 100, TipJarError::MemoTooLong);

        let tip_jar = &mut ctx.accounts.tipjar;
        let sender = &ctx.accounts.sender;
        
        // Check if tip jar is active
        if !tip_jar.is_active {
            // Emit an event for the refund
            emit!(TipRefunded {
                tipjar: tip_jar.key(),
                sender: sender.key(),
                lamports: amount,
                timestamp: Clock::get()?.unix_timestamp as u64,
            });
            
            return Ok(());
        }

        // Check privacy settings
        if tip_jar.is_private {
            // Block if the sender is not the owner
            require_keys_eq!(sender.key(), tip_jar.owner, TipJarError::Unauthorized);
        }

        // Create the new tip
        let new_tip = Tip {
            sender: sender.key(),
            amount,
            visibility,
            memo: memo.clone(),
            timestamp: Clock::get()?.unix_timestamp as u64,
        };
        
        // Store the tip using circular buffer to maintain fixed size history
        if tip_jar.tips_history.len() < TipJar::MAX_TIP_HISTORY_LEN {
            tip_jar.tips_history.push(new_tip.clone());
        } else {
            let index = (tip_jar.last_tip_index as usize) % TipJar::MAX_TIP_HISTORY_LEN;
            tip_jar.tips_history[index] = new_tip.clone();
            tip_jar.last_tip_index = ((tip_jar.last_tip_index as usize + 1) % TipJar::MAX_TIP_HISTORY_LEN) as u16;
        }
        
        // Increment total tips counter
        tip_jar.total_tips_count += 1;

        // Transfer SOL from sender to tip jar using the Solana System Program
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &sender.key(),
            &tip_jar.key(),
            amount,
        );
    
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                sender.to_account_info(),
                tip_jar.to_account_info(),
            ],
        )?;

        // Update the total_received in the TipJar
        tip_jar.total_received += amount;

        // Emit an event logging the tip info
        emit!(TipSent {
            sender: sender.key(),
            receiver: tip_jar.key(),
            amount,
            memo,
            visibility,
        });
    
        // Check if goal has been reached
        if tip_jar.total_received >= tip_jar.goal {
            emit!(GoalReached {
                tipjar: tip_jar.key(),
                goal: tip_jar.goal,
                total_received: tip_jar.total_received,
            });
        }
        
        Ok(())
    }

    /// Helper function to get tip history with pagination
    pub fn get_tip_history(tip_jar: &TipJar, page: u32, page_size: u32) -> Vec<&Tip> {
        let start = (page * page_size) as usize;
        let end = std::cmp::min(start + page_size as usize, tip_jar.tips_history.len());
        
        if start >= tip_jar.tips_history.len() {
            return vec![];
        }
        
        // Return a slice of the tips history
        tip_jar.tips_history[start..end].iter().collect()
    }

    /// Emits stats about a tip jar without fetching all tips
    pub fn get_tip_stats(ctx: Context<GetTipStats>) -> Result<()> {
        let tip_jar = &ctx.accounts.tipjar;
        
        emit!(TipJarStats {
            tipjar: tip_jar.key(),
            total_tips: tip_jar.total_tips_count,
            total_received: tip_jar.total_received,
            is_active: tip_jar.is_active,
            goal_percentage: if tip_jar.goal > 0 {
                (tip_jar.total_received * 100) / tip_jar.goal
            } else {
                0
            },
        });
        
        Ok(())
    }

    /// Clears tip history while maintaining total count
    pub fn clear_tip_history(ctx: Context<ClearTipHistory>) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let owner = &ctx.accounts.owner;
        
        // Only owner can clear history
        require_keys_eq!(tip_jar.owner, owner.key(), TipJarError::Unauthorized);
        
        // Clear tips history but maintain total count
        tip_jar.tips_history.clear();
        tip_jar.last_tip_index = 0;
        
        msg!("Tip history cleared while maintaining total count of {}", tip_jar.total_tips_count);
        
        Ok(())
    }

    /// Toggles the active status of a tip jar
    pub fn toggle_tipjar_status(ctx: Context<ToggleTipJarStatus>) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let signer = &ctx.accounts.owner;
    
        // Security: Only the owner can pause/resume
        require_keys_eq!(tip_jar.owner, signer.key(), TipJarError::Unauthorized);
    
        // Calculate the new status (opposite of current)
        let new_status = !tip_jar.is_active;
        
        // Prevent redundant operations
        require!(tip_jar.is_active != new_status, TipJarError::RedundantStatusChange);
    
        // Flip the active flag
        tip_jar.is_active = new_status;
    
        emit!(TipJarStatusChanged {
            tipjar: tip_jar.key(),
            is_active: tip_jar.is_active,
        });
        Ok(())
    }

    /// Updates tip jar metadata (description, category, goal)
    pub fn update_tipjar(ctx: Context<UpdateTipJar>, new_description: String, new_category: String, new_goal: u64) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let signer = &ctx.accounts.owner;
    
        // Only the owner can update the tip jar
        require_keys_eq!(tip_jar.owner, signer.key(), TipJarError::Unauthorized);
    
        // Apply updates
        tip_jar.description = new_description;
        tip_jar.category = new_category;
        tip_jar.goal = new_goal;
    
        msg!("TipJar updated successfully.");
    
        Ok(())
    }

    /// Allows the owner to withdraw funds from the tip jar
    pub fn withdraw_tip(ctx: Context<WithdrawTip>, amount: u64) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let signer = &ctx.accounts.owner;

        // Only the owner can withdraw
        require_keys_eq!(tip_jar.owner, signer.key(), TipJarError::Unauthorized);

        // Ensure there are enough funds to withdraw
        require!(tip_jar.total_received >= amount, TipJarError::InsufficientFunds);

        // Set a withdrawal limit for security
        let withdraw_limit = 1000; // limit of 1000 SOL per withdrawal
        require!(amount <= withdraw_limit, TipJarError::WithdrawalLimitExceeded);

        // Prepare the transfer instruction from tip jar to owner
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &tip_jar.key(),
            &signer.key(),
            amount,
        );
        
        // Execute the transfer
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                tip_jar.to_account_info(),
                signer.to_account_info(),
            ],
        )?;

        // Update the total_received in the TipJar
        tip_jar.total_received -= amount;

        msg!("Withdrawal successful. Amount withdrawn: {}", amount);

        Ok(())
    }

    /// Pauses a tip jar (sets is_active to false)
    pub fn pause_tipjar(ctx: Context<PauseTipJar>) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let owner = &ctx.accounts.owner;
    
        // Ensure the caller is the owner of the tip jar
        require_keys_eq!(tip_jar.owner, owner.key(), TipJarError::Unauthorized);
    
        // Set the TipJar to inactive (paused)
        tip_jar.is_active = false;
    
        msg!("TipJar paused by the owner.");
    
        Ok(())
    }

    /// Resumes a paused tip jar (sets is_active to true)
    pub fn resume_tipjar(ctx: Context<ResumeTipJar>) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let owner = &ctx.accounts.owner;
    
        // Ensure the caller is the owner of the tip jar
        require_keys_eq!(tip_jar.owner, owner.key(), TipJarError::Unauthorized);
    
        // Set the TipJar to active (resumed)
        tip_jar.is_active = true;
    
        msg!("TipJar resumed by the owner.");
    
        Ok(())
    }

    /// Closes a tip jar, transferring remaining funds to owner and recovering rent
    pub fn close_tipjar(ctx: Context<CloseTipJar>) -> Result<()> {
        let tip_jar = &mut ctx.accounts.tipjar;
        let owner = &ctx.accounts.owner;
    
        // Ensure the caller is the owner of the tip jar
        require_keys_eq!(tip_jar.owner, owner.key(), TipJarError::Unauthorized);

        // Get the remaining amount to transfer
        let amount_to_transfer = tip_jar.total_received;
    
        if amount_to_transfer > 0 {
            // Transfer any remaining SOL to the owner
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &tip_jar.key(),
                &owner.key(),
                amount_to_transfer,
            );
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    tip_jar.to_account_info(),
                    owner.to_account_info(),
                ],
            )?;
        }

        // Close the TipJar account and recover rent
        msg!("Closing TipJar and transferring {} SOL to owner", amount_to_transfer);

        // Close account and send lamports to owner
        let tip_jar_account_info = tip_jar.to_account_info();
        let dest_account_info = owner.to_account_info();
        **dest_account_info.lamports.borrow_mut() += **tip_jar_account_info.lamports.borrow();
        **tip_jar_account_info.lamports.borrow_mut() = 0;
        
        Ok(())
    }
}

// Context struct for initializing a tip jar
// This defines what accounts are required for the instruction
#[derive(Accounts)]
#[instruction(description: String, category: String)]
pub struct InitializeTipJar<'info> {
    #[account(
        init,                              // Create a new account
        payer = user,                      // User pays for account creation
        space = 8 + TipJar::LEN,           // Allocate space for the account
        seeds = [b"tipjar", user.key().as_ref()], // PDA seeds for deterministic address
        bump                               // Add bump to ensure unique address
    )]
    pub tipjar: Account<'info, TipJar>,    // The account to create
    
    #[account(mut)]                        // Mark as mutable because we'll deduct rent
    pub user: Signer<'info>,               // User must sign the transaction
    
    pub system_program: Program<'info, System>, // Required for account creation
}

// Context struct for sending a tip
#[derive(Accounts)]
#[instruction(memo: String)]
pub struct SendTip<'info> {
    #[account(mut)]                        // Mutable because we're updating it
    pub tipjar: Account<'info, TipJar>,    // The target TipJar to receive the tip

    #[account(mut)]                        // Mutable because we're deducting SOL
    pub sender: Signer<'info>,             // The user sending the tip

    pub system_program: Program<'info, System>, // Required for transferring SOL
}

// Context struct for getting tip statistics
#[derive(Accounts)]
pub struct GetTipStats<'info> {
    pub tipjar: Account<'info, TipJar>,    // The tip jar to get stats for
}

// Context struct for clearing tip history
#[derive(Accounts)]
pub struct ClearTipHistory<'info> {
    #[account(mut, has_one = owner)]       // Mutable with owner validation
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,              // Owner must sign the transaction
}

// Context struct for toggling tip jar status
#[derive(Accounts)]
pub struct ToggleTipJarStatus<'info> {
    #[account(mut, has_one = owner)]       // has_one ensures the owner field matches
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// Context struct for updating tip jar details
#[derive(Accounts)]
pub struct UpdateTipJar<'info> {
    #[account(mut, has_one = owner)]
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// Context struct for pausing a tip jar
#[derive(Accounts)]
pub struct PauseTipJar<'info> {
    #[account(mut, has_one = owner)]
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// Context struct for resuming a tip jar
#[derive(Accounts)]
pub struct ResumeTipJar<'info> {
    #[account(mut, has_one = owner)]
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
}

// Context struct for withdrawing tips
#[derive(Accounts)]
pub struct WithdrawTip<'info> {
    #[account(mut, has_one = owner)]
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Context struct for closing a tip jar
#[derive(Accounts)]
pub struct CloseTipJar<'info> {
    #[account(mut, has_one = owner, close = owner)] // close = owner transfers rent to owner
    pub tipjar: Account<'info, TipJar>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Event emitted when a tip is sent
#[event]
pub struct TipSent {
    pub sender: Pubkey,                    // Address of the person sending the tip
    pub receiver: Pubkey,                  // Address of the TipJar
    pub amount: u64,                       // Amount of SOL sent
    pub memo: String,                      // Message attached to the tip
    pub visibility: Visibility,            // Whether the tip is public or anonymous
}

// Event emitted when a tip jar status changes
#[event]
pub struct TipJarStatusChanged {
    pub tipjar: Pubkey,
    pub is_active: bool,
}

// Event emitted when a goal is reached
#[event]
pub struct GoalReached {
    pub tipjar: Pubkey,
    pub goal: u64,
    pub total_received: u64,
}

// Event emitted when a tip is refunded
#[event]
pub struct TipRefunded {
    pub tipjar: Pubkey,
    pub sender: Pubkey,
    pub lamports: u64,
    pub timestamp: u64,
}

// Event emitted for tip jar statistics
#[event]
pub struct TipJarStats {
    pub tipjar: Pubkey,
    pub total_tips: u32,
    pub total_received: u64, 
    pub is_active: bool,
    pub goal_percentage: u64,
}

// Error enum for the program
#[error]
pub enum TipJarError {
    #[msg("The TipJar is currently inactive")]
    InactiveTipJar,

    #[msg("The tip amount must be greater than zero")]
    InvalidAmount,

    #[msg("Invalid visibility option selected")]
    InvalidVisibility,

    #[msg("Insufficient funds in the tip jar for this operation")]
    InsufficientFunds,

    #[msg("Only the owner can perform this action")]
    Unauthorized,

    #[msg("Transaction failed due to an unexpected condition")]
    UnexpectedTransactionFailure,
    
    #[msg("Amount exceeds withdrawal limit of 1000 SOL")]
    WithdrawalLimitExceeded,

    #[msg("Memo is too long (maximum 100 characters)")]
    MemoTooLong,

    #[msg("Tip jar is already in the requested state")]
    RedundantStatusChange,
    
    #[msg("Cannot initialize tip jar with zero or negative goal")]
    InvalidGoal,
    
    #[msg("Description is too long (maximum 200 characters)")]
    DescriptionTooLong,
    
    #[msg("Category is too long (maximum 100 characters)")]
    CategoryTooLong,
    
    #[msg("Tip history capacity exceeded")]
    TipHistoryFull,
    
    #[msg("Cannot close a tip jar with active balance")]
    NonEmptyJarClosure,
    
    #[msg("Operation not allowed during active tips")]
    OperationDuringActiveTips,
}