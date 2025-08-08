use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, MintTo};
use crate::state::*;
use crate::constants::*;
use crate::errors::*;

#[derive(Accounts)]
#[instruction(report_id: [u8; 32], description: String, proof_of_concept: String, severity: BugSeverity)]
pub struct SubmitBugReport<'info> {
    #[account(
        init,
        payer = reporter,
        space = BugReport::MAX_LEN,
    seeds = [BUG_REPORT_SEED, &report_id],
        bump
    )]
    pub bug_report: Account<'info, BugReport>,

    #[account(
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(mut)]
    pub reporter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(report_id: [u8; 32], is_valid: bool)]
pub struct VerifyBugReport<'info> {
    #[account(
        mut,
        seeds = [BUG_REPORT_SEED, &report_id],
        bump
    )]
    pub bug_report: Account<'info, BugReport>,

    #[account(
        mut,
        seeds = [GAME_STATE_SEED],
        bump = game_state.bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        mut,
        seeds = [GLORY_MINT_SEED],
        bump
    )]
    pub glory_mint: Account<'info, anchor_spl::token::Mint>,

    #[account(
        mut,
        associated_token::mint = glory_mint,
        associated_token::authority = reporter_key
    )]
    pub reporter_glory_account: Account<'info, TokenAccount>,

    /// CHECK: This is the reporter's pubkey from the bug report
    pub reporter_key: AccountInfo<'info>,

    #[account(
        mut,
        constraint = admin.key() == game_state.admin
    )]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn submit_report_handler(
    ctx: Context<SubmitBugReport>,
    report_id: [u8; 32],
    description: String,
    proof_of_concept: String,
    severity: BugSeverity,
) -> Result<()> {
    let bug_report = &mut ctx.accounts.bug_report;
    let clock = Clock::get()?;

    // Validate description and PoC length
    require!(description.len() <= 256, GameError::InvalidAmount);
    require!(proof_of_concept.len() <= 512, GameError::InvalidAmount);

    // Initialize bug report
    bug_report.reporter = ctx.accounts.reporter.key();
    bug_report.report_id = report_id;
    bug_report.severity = severity;
    bug_report.description = description;
    bug_report.proof_of_concept = proof_of_concept;
    bug_report.timestamp = clock.unix_timestamp;
    bug_report.is_verified = false;
    bug_report.is_paid = false;
    bug_report.bounty_amount = severity.get_bounty_amount();
    bug_report.verifier = None;

    msg!("Bug report submitted by {} with severity {:?}",
         ctx.accounts.reporter.key(),
         severity);

    Ok(())
}

pub fn verify_report_handler(
    ctx: Context<VerifyBugReport>,
    report_id: [u8; 32],
    is_valid: bool,
) -> Result<()> {
    let bug_report = &mut ctx.accounts.bug_report;
    let game_state = &mut ctx.accounts.game_state;

    // Check if report exists and hasn't been verified yet
    require!(!bug_report.is_verified, GameError::BugReportAlreadyVerified);
    require!(!bug_report.is_paid, GameError::BugReportAlreadyPaid);

    // Mark as verified
    bug_report.is_verified = true;
    bug_report.verifier = Some(ctx.accounts.admin.key());

    if is_valid {
    // Ensure reporter account matches report
    require!(ctx.accounts.reporter_key.key() == bug_report.reporter, GameError::Unauthorized);
        // Enforce GLORY supply cap for bounty
        require!(
            game_state.total_glory_minted.saturating_add(bug_report.bounty_amount) <= GLORY_SUPPLY_CAP,
            GameError::InvalidAmount
        );
        // Pay bounty by minting GLORY tokens
        let game_state_key = game_state.key();
        let seeds = &[
            GAME_STATE_SEED,
            &[game_state.bump],
        ];
        let signer = &[&seeds[..]];

        let mint_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.glory_mint.to_account_info(),
                to: ctx.accounts.reporter_glory_account.to_account_info(),
                authority: game_state.to_account_info(),
            },
            signer,
        );
        token::mint_to(mint_ctx, bug_report.bounty_amount)?;

    bug_report.is_paid = true;
    game_state.total_glory_minted = game_state.total_glory_minted.saturating_add(bug_report.bounty_amount);

        msg!("Bug bounty paid: {} GLORY to {}",
             bug_report.bounty_amount,
             bug_report.reporter);
    } else {
        msg!("Bug report rejected for: {}", bug_report.reporter);
    }

    Ok(())
}
