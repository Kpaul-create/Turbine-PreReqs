use anchor_lang::prelude::*;

declare_id!("TRBQyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM");

#[program]
pub mod q3_pre_reqs_rs {
    use super::*;

    /// Initialize a registration (application) account for a user.
    ///
    /// Records the user's GitHub username. This is the RS-side prerequisite
    /// registration. Each wallet can register only once.
    pub fn initialize(ctx: Context<Initialize>, github: String) -> Result<()> {
        let application_account = &mut ctx.accounts.account;
        application_account.user = ctx.accounts.user.key();
        application_account.bump = ctx.bumps.account;
        application_account.pre_req_ts = false;
        application_account.pre_req_rs = true;
        application_account.github = github;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut, signer)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"prereqs", user.key().as_ref()],
        bump,
        space = 8 + 32 + 1 + 1 + 1 + 200
    )]
    pub account: Account<'info, ApplicationAccount>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct ApplicationAccount {
    pub user: Pubkey,
    pub bump: u8,
    pub pre_req_ts: bool,
    pub pre_req_rs: bool,
    pub github: String,
}
