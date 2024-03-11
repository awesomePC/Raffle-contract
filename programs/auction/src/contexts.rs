use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Token, Mint, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use std::mem::size_of;

use crate::account::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct CreateUserContext<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  #[account(init, seeds = [
    b"user_", 
    owner.key().as_ref()], 
    bump, 
    payer = owner, 
    space = size_of::<User>() + 8
  )]
  pub user: AccountLoader<'info, User>,
  pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CreateAdminContext<'info> {
  #[account(mut, constraint = owner.key() == ADMIN_KEY)]
  pub owner: Signer<'info>,
  #[account(mut)]
    /// CHECK: it's not dangerous
  pub admin: AccountInfo<'info>,
  #[account(mut)]
  pub user: AccountLoader<'info, User>,
  pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct DeleteAdminContext<'info> {
  #[account(mut, constraint = owner.key() == ADMIN_KEY)]
  pub owner: Signer<'info>,
  #[account(mut)]
    /// CHECK: it's not dangerous
  pub admin: AccountInfo<'info>,
  #[account(mut)]
  pub user: AccountLoader<'info, User>,
  pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct FeeContext<'info> {
    #[account(init_if_needed, seeds = [b"auction_fee"], bump, payer = admin, space = 8 + 304)]
    pub fee_account: Account<'info, AuctionState>,
    #[account(mut, constraint = admin.key() == ADMIN_KEY)]
    admin: Signer<'info>,
    system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(auction_id: u64)]
pub struct CreateAuctionContext<'info> {
  // #[account(mut, constraint = admin.key() == ADMIN_KEY)]
  #[account(mut)]
  pub admin: Signer<'info>,
  #[account(init, seeds = [
    POOL_SEED.as_bytes(), 
    &auction_id.to_le_bytes(), 
    mint.key().as_ref()], 
    bump, 
    payer = admin, 
    space = size_of::<Pool>() + 8
  )]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)]  //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  pub fee_account: Account<'info, AuctionState>,
  pub mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == mint.key() && ata_from.owner == admin.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = admin,
    associated_token::mint = mint,
    associated_token::authority = pool
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>
}

impl<'info> CreateAuctionContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
          from: self.ata_from.to_account_info().clone(),
          to: self.ata_to.to_account_info().clone(),
          authority: self.admin.to_account_info().clone(),
      };
      CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct EditAuctionContext<'info> {
  #[account(mut, constraint = admin.key() == ADMIN_KEY)]
  pub admin: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>
}

#[derive(Accounts)]
pub struct DeleteAuctionContext<'info> {
  #[account(mut, constraint = admin.key() == ADMIN_KEY)]
  pub admin: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  pub mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = admin,
    associated_token::mint = mint,
    associated_token::authority = admin
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent>
}

impl<'info> DeleteAuctionContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
          from: self.ata_from.to_account_info().clone(),
          to: self.ata_to.to_account_info().clone(),
          authority: self.pool.to_account_info().clone(),
      };
      CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct CreateBidContext<'info> {
  #[account(mut)]
  pub bidder: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)] //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  pub fee_account: Account<'info, AuctionState>,
  // #[account(constraint = pay_mint.key() == PAY_TOKEN)]
  #[account(mut)] 
  pub pay_mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == pay_mint.key() && ata_from.owner == bidder.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = bidder,
    associated_token::mint = pay_mint,
    associated_token::authority = pool
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

impl<'info> CreateBidContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
          from: self.ata_from.to_account_info().clone(),
          to: self.ata_to.to_account_info().clone(),
          authority: self.bidder.to_account_info().clone(),
      };
      CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}


#[derive(Accounts)]
pub struct UpdateBidContext<'info> {
  #[account(mut)]
  pub bidder: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)]  //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  // #[account(constraint = pay_mint.key() == PAY_TOKEN)]
  #[account(mut)] 
  pub pay_mint: Account<'info, Mint>,
  #[account(
    init_if_needed,
    payer = bidder,
    associated_token::mint = pay_mint,
    associated_token::authority = bidder
  )]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(mut, constraint = ata_to.mint == pay_mint.key() && ata_to.owner == pool.key())]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

impl<'info> UpdateBidContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
      let cpi_accounts = Transfer {
          from: self.ata_from.to_account_info().clone(),
          to: self.ata_to.to_account_info().clone(),
          authority: self.bidder.to_account_info().clone(),
      };
      CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }

  pub fn reverse_transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_to.to_account_info().clone(),
        to: self.ata_from.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct CancelBidContext<'info> {
  #[account(mut)]
  pub bidder: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)]  //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  // #[account(constraint = pay_mint.key() == PAY_TOKEN)]
  #[account(mut)] 
  pub pay_mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == pay_mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = bidder,
    associated_token::mint = pay_mint,
    associated_token::authority = bidder
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 

}

impl<'info> CancelBidContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_from.to_account_info().clone(),
        to: self.ata_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}


#[derive(Accounts)]
pub struct ClaimBidContext<'info> {
  #[account(mut)]
  pub bidder: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)] //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  pub fee_account: Account<'info, AuctionState>,
  // #[account(constraint = pay_mint.key() == PAY_TOKEN)]
  #[account(mut)] 
  pub pay_mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == pay_mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = bidder,
    associated_token::mint = pay_mint,
    associated_token::authority = bidder
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

impl<'info> ClaimBidContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_from.to_account_info().clone(),
        to: self.ata_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct ClaimPrizeContext<'info> {
  /// CHECK: it's not dangerous
  #[account(mut, constraint = community.key() == COMMUNITY_KEY)]
  pub community: AccountInfo<'info>,
  #[account(mut)]
  pub bidder: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)] //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  pub mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = bidder,
    associated_token::mint = mint,
    associated_token::authority = bidder
  )]
  pub ata_to: Account<'info, TokenAccount>,
  // #[account(constraint = pay_mint.key() == PAY_TOKEN)]  
  // pub pay_mint: Account<'info, Mint>,
  // #[account(mut, constraint = token_from.mint == pay_mint.key() && token_from.owner == pool.key())]
  #[account(mut, constraint = token_from.owner == pool.key())]
  pub token_from: Account<'info, TokenAccount>,
  // #[account(
  //   init_if_needed,
  //   payer = bidder,
  //   associated_token::mint = pay_mint,
  //   associated_token::authority = community
  // )]
  #[account(mut)]
  pub token_to: Box<Account<'info, TokenAccount>>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

 

impl<'info> ClaimPrizeContext<'info> {
  pub fn transfer_nft_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_from.to_account_info().clone(),
        to: self.ata_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }

  pub fn transfer_ft_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.token_from.to_account_info().clone(),
        to: self.token_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct SendBackNftContext<'info> {
  #[account(mut)]
  pub partner: Signer<'info>,
  /// CHECK: it's not dangerous
  #[account(mut)]
  pub admin: AccountInfo<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  pub mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = partner,
    associated_token::mint = mint,
    associated_token::authority = admin
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

impl<'info> SendBackNftContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_from.to_account_info().clone(),
        to: self.ata_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct SendBackFtContext<'info> {
  #[account(mut)]
  pub partner: Signer<'info>,
   /// CHECK: it's not dangerous
   #[account(mut)]
   pub bidder: AccountInfo<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  #[account(mut)] //  constraint = treasury.key() == WITHDRAW_KEY
  pub treasury: AccountInfo<'info>,
  pub pay_mint: Account<'info, Mint>,
  #[account(mut, constraint = ata_from.mint == pay_mint.key() && ata_from.owner == pool.key())]
  pub ata_from: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = partner,
    associated_token::mint = pay_mint,
    associated_token::authority = bidder
  )]
  pub ata_to: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>,
  pub rent: Sysvar<'info, Rent> 
}

impl<'info> SendBackFtContext<'info> {
  pub fn transfer_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    let cpi_accounts = Transfer {
        from: self.ata_from.to_account_info().clone(),
        to: self.ata_to.to_account_info().clone(),
        authority: self.pool.to_account_info().clone(),
    };
    CpiContext::new(self.token_program.to_account_info().clone(), cpi_accounts)
  }
}

#[derive(Accounts)]
pub struct SetWinnerContext<'info> {
  #[account(mut)]
  pub partner: Signer<'info>,
  #[account(mut)]
  pub pool: AccountLoader<'info, Pool>
}