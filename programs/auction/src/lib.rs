use anchor_lang::prelude::*;
use anchor_spl::token::{self};

pub mod contexts;
pub mod utils;
pub mod constants;
pub mod account;
pub mod errors;

use contexts::*;
use utils::*;
use errors::*;
use constants::*;

declare_id!("HBCz3iBqcGk45uWLq6MRSpGtJSyazTyERZm3ExGqzXgx");

#[program]
pub mod auction {
    use super::*;

    use anchor_lang::AccountsClose;

    pub fn create_user(
        ctx: Context<CreateUserContext>
    ) -> Result<()> {
        let mut a_user = ctx.accounts.user.load_init()?;
        a_user.count = 0;

        Ok(())
    }

    pub fn create_admin(
        ctx: Context<CreateAdminContext>
    ) -> Result<()> {
        let mut a_user = ctx.accounts.user.load_mut()?;
        let a_admin = &ctx.accounts.admin;

        require!(
            (a_user.count as usize) < 100, 
            AuctionError::OverAdminMaxCount
        );

        let result: bool = a_user.add_admin(a_admin.to_account_info().key())?;
        require!(result, AuctionError::CreateAdminError);

        a_user.count += 1;

        Ok(())
    }

    pub fn remove_admin(
        ctx: Context<DeleteAdminContext>
    ) -> Result<()> {
        let mut a_user = ctx.accounts.user.load_mut()?;
        let a_admin = &ctx.accounts.admin;

        let result: bool = a_user.delete_admin(a_admin.to_account_info().key())?;
        require!(result, AuctionError::DeleteAdminError);

        a_user.count -= 1;

        Ok(())
    }
    
    pub fn set_fee(ctx: Context<FeeContext>, fee1: u64, fee2: u64, fee3:u64) -> Result<()> {
        let contract_data = &mut ctx.accounts.fee_account;

        contract_data.create_fee = fee1;
        contract_data.claim_fee = fee2;
        contract_data.ticket_fee = fee3;
        
        Ok(())
    }

    pub fn create_auction(
        ctx: Context<CreateAuctionContext>, 
        auction_id: u64, 
        start_time: u32,
        end_time: u32, 
        min_price: u64,
        bid_increment: u64
    ) -> Result<()> {
        let mut a_pool = ctx.accounts.pool.load_init()?;

        require!(
            start_time < end_time,
            AuctionError::StartedAuction
        );

        let fee_account = &mut ctx.accounts.fee_account;

        // Must send the fee amount from admin to treasury 
        transfer_lamports(
            &ctx.accounts.admin,
            &ctx.accounts.treasury.to_account_info(),
            &ctx.accounts.system_program,
            fee_account.create_fee
        )?;

        let a_mint = &ctx.accounts.mint;

        a_pool.auction_id = auction_id;
        a_pool.start_time = start_time;
        a_pool.end_time = end_time;
        a_pool.mint = a_mint.to_account_info().key();
        a_pool.min_price = min_price;
        a_pool.bid_increment = bid_increment;
        a_pool.count = 0;
        a_pool.state = 0;
        token::transfer(ctx.accounts.transfer_context(), 1)?;
        Ok(())
    }

    pub fn edit_auction(
        ctx: Context<EditAuctionContext>, 
        start_time: u32,
        end_time: u32, 
        min_price: u64,
        bid_increment: u64,
    ) -> Result<()> {
        let mut a_pool = ctx.accounts.pool.load_mut()?;
        let current_time = get_current_time()?;

        require!(
            start_time < end_time,
            AuctionError::StartedAuction
        );
        
        require!(
            current_time < a_pool.start_time,
            AuctionError::StartedAuction
        );

        a_pool.start_time = start_time;
        a_pool.end_time = end_time;
        a_pool.min_price = min_price;
        a_pool.bid_increment = bid_increment;
        Ok(())
    }

    pub fn delete_auction(
        ctx: Context<DeleteAuctionContext>
    ) -> Result<()> {
        {
            let a_pool = ctx.accounts.pool.load()?;
            let a_mint = &ctx.accounts.mint;

            let current_time = get_current_time()?;

            require!(
                current_time < a_pool.start_time || ( a_pool.count == 0 &&  current_time > a_pool.end_time) ,
                AuctionError::StartedAuction
            );

            let clone_auction_id = a_pool.auction_id;
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &clone_auction_id.to_le_bytes(), a_mint.to_account_info().key.as_ref(), &[bump]];
            let signer = &[&seeds[..]];

            token::transfer(
                ctx.accounts.transfer_context().with_signer(signer), 
                1
            )?;
        }

        {
            let a_admin = &ctx.accounts.admin;
            ctx.accounts.pool.close(a_admin.to_account_info())?;
        }
        Ok(())
    }

    pub fn create_bid(ctx: Context<CreateBidContext>, price: u64, nft_count: u32) -> Result<()> {
        let mut a_pool = ctx.accounts.pool.load_mut()?;
        let a_bidder = &ctx.accounts.bidder;

        let current_time = get_current_time()?;

        require!(price >= a_pool.min_price + a_pool.bid_increment, AuctionError::InvalidPrice);
        require!(
            current_time >= a_pool.start_time && 
            current_time <= a_pool.end_time, 
            AuctionError::OutOfAuction
        );
        require!(
            (a_pool.count as usize) < MAX_BID_COUNT, 
            AuctionError::OverMaxCount
        );

        if a_pool.mint.to_string() == SOL_KEY.to_string() {
            transfer_lamports(
                &ctx.accounts.bidder,
                &ctx.accounts.treasury.to_account_info(),
                &ctx.accounts.system_program,
                price
            )?;
        } else {
            token::transfer(ctx.accounts.transfer_context(), price)?;
        }


        let fee_account = &mut ctx.accounts.fee_account;

        // Must send the fee amount from admin to treasury 
        transfer_lamports(
            &ctx.accounts.bidder,
            &ctx.accounts.treasury.to_account_info(),
            &ctx.accounts.system_program,
            fee_account.ticket_fee
        )?;

        let result: bool = a_pool.add_bid(a_bidder.to_account_info().key(), price)?;

        require!(result == true, AuctionError::CreateBidError);

        a_pool.count += 1;
        a_pool.min_price = price;
        Ok(())
    }

    pub fn update_bid(ctx: Context<UpdateBidContext>, price: u64, nft_count: u32) -> Result<()> {
        let a_bidder = &ctx.accounts.bidder;
        
        let current_time = get_current_time()?;
        let mut old_price: u64 = 0;
        let mut a_pool = ctx.accounts.pool.load_mut()?;
        
        {
            require!(price >= a_pool.min_price + a_pool.bid_increment, AuctionError::InvalidPrice);
            require!(
                current_time >= a_pool.start_time && 
                current_time <= a_pool.end_time, 
                AuctionError::OutOfAuction
            );
            
            old_price = a_pool.update_bid(a_bidder.to_account_info().key(), price)?;
            require!(old_price != 0, AuctionError::UpdateBidError);
        }

        {

            if price > old_price {
                if a_pool.mint.to_string() == SOL_KEY.to_string() {
                    transfer_lamports(
                        &ctx.accounts.bidder,
                        &ctx.accounts.treasury.to_account_info(),
                        &ctx.accounts.system_program,
                        price - old_price
                    )?;
                } else {
                    token::transfer(ctx.accounts.transfer_context(), price - old_price)?;
                }
            }
           
        }

        a_pool.min_price = price;

        Ok(())
    }

    pub fn cancel_bid(ctx: Context<CancelBidContext>) -> Result<()> {
        let mut price: u64 = 0;
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
            let a_bidder = &ctx.accounts.bidder;

            let current_time = get_current_time()?;

            require!(
                current_time >= a_pool.start_time && 
                current_time <= a_pool.end_time, 
                AuctionError::OutOfAuction
            );

            price = a_pool.cancel_bid(a_bidder.to_account_info().key())?;
            require!(price > 0, AuctionError::CancelBidError);
            a_pool.count -= 1;

        }
        {
            let a_pool = ctx.accounts.pool.load()?;
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &a_pool.auction_id.to_le_bytes(), a_pool.mint.as_ref(), &[bump]];
            let signer = &[&seeds[..]];

            if a_pool.mint.to_string() == SOL_KEY.to_string() {
                transfer_lamports_from_treasury(
                    &ctx.accounts.treasury,
                    &ctx.accounts.bidder,
                    ctx.program_id,
                    price
                )?;
            } else {
                token::transfer(
                    ctx.accounts.transfer_context().with_signer(signer), 
                    price
                )?;
            }
        }

        Ok(())
    }

    pub fn claim_bid(ctx: Context<ClaimBidContext>) -> Result<()> {
        let (mut price, mut claimed) = (0, false);
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
            let a_bidder = &ctx.accounts.bidder;
    
            let current_time = get_current_time()?;
    
            require!(
                current_time >= a_pool.end_time, 
                AuctionError::NotFinishAuction
            );

            (price, claimed) = a_pool.claim_bid(a_bidder.to_account_info().key())?;
            require!(price > 0, AuctionError::ClaimBidError);
            require!(!claimed, AuctionError::AlreadyClaimed);
        }

        let fee_account = &mut ctx.accounts.fee_account;

        // Must send the fee amount from admin to treasury 
        transfer_lamports(
            &ctx.accounts.bidder,
            &ctx.accounts.treasury.to_account_info(),
            &ctx.accounts.system_program,
            fee_account.claim_fee
        )?;

        {
            let a_pool = ctx.accounts.pool.load()?;
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &a_pool.auction_id.to_le_bytes(), a_pool.mint.as_ref(), &[bump]];
            let signer = &[&seeds[..]];

            if a_pool.mint.to_string() == SOL_KEY.to_string() {
                transfer_lamports_from_treasury(
                    &ctx.accounts.treasury,
                    &ctx.accounts.bidder,
                    ctx.program_id,
                    price
                )?;
            } else {
                token::transfer(
                    ctx.accounts.transfer_context().with_signer(signer), 
                    price
                )?;
            }
    
            Ok(())
        }
    }

    pub fn claim_prize(ctx: Context<ClaimPrizeContext>) -> Result<()> {
        let mut price = 0;
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
            let a_bidder = &ctx.accounts.bidder;
    
            let current_time = get_current_time()?;
    
            require!(
                current_time >= a_pool.end_time, 
                AuctionError::NotFinishAuction
            );
    
            require!(
                a_pool.state != 2, 
                AuctionError::AlreadyClaimedPrize
            );
    
            a_pool.state = 2;

            price = a_pool.get_bid_price(a_bidder.to_account_info().key())?;
            require!(price > 0, AuctionError::GetPriceError);
    
            let result = a_pool.claim_prize(a_bidder.to_account_info().key())?;
            require!(result, AuctionError::NotWinner);
        }

        {
            let a_pool = ctx.accounts.pool.load()?;
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &a_pool.auction_id.to_le_bytes(), a_pool.mint.as_ref(), &[bump]];
            let signer = &[&seeds[..]];
    
            token::transfer(
                ctx.accounts.transfer_nft_context().with_signer(signer), 
                1
            )?;

            if a_pool.mint.to_string() == SOL_KEY.to_string() {
                transfer_lamports_from_treasury(
                    &ctx.accounts.treasury,
                    &ctx.accounts.bidder,
                    ctx.program_id,
                    price
                )?;
            } else {
                token::transfer(
                    ctx.accounts.transfer_ft_context().with_signer(signer), 
                    price
                )?;
            }
    
        }
        Ok(())
    }

    pub fn send_back_nft(ctx: Context<SendBackNftContext>) -> Result<()> {
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
    
            let current_time = get_current_time()?;
    
            require!(
                current_time >= a_pool.end_time, 
                AuctionError::NotFinishAuction
            );

            require!(
                a_pool.state >= 1, 
                AuctionError::NotFinishAuction
            );

            let result = a_pool.exist_winner()?;
            require!(!result, AuctionError::AlreadySetWinner);
    
            a_pool.state = 3;
    
        }

        {
            let a_pool = ctx.accounts.pool.load()?;
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &a_pool.auction_id.to_le_bytes(), a_pool.mint.as_ref(), &[bump]];
            let signer = &[&seeds[..]];
    
            token::transfer(
                ctx.accounts.transfer_context().with_signer(signer), 
                1
            )?;
    
        }
        Ok(())
    }

    pub fn send_back_ft(ctx: Context<SendBackFtContext>) -> Result<()> {
        let mut price = 0;
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
            let a_bidder = &ctx.accounts.bidder;

            let current_time = get_current_time()?;
    
            require!(
                current_time >= a_pool.end_time, 
                AuctionError::NotFinishAuction
            );

            require!(
                a_pool.state >= 1, 
                AuctionError::NotFinishAuction
            );

            let result = a_pool.exist_winner()?;
            require!(!result, AuctionError::AlreadySetWinner);

            price = a_pool.get_bid_price(a_bidder.to_account_info().key())?;

            a_pool.state = 4; 
        }
        {
            let a_pool = ctx.accounts.pool.load()?;
            
            let (_pool, bump) = Pubkey::find_program_address(
                &[POOL_SEED.as_ref(), 
                &a_pool.auction_id.to_le_bytes(), 
                a_pool.mint.as_ref()], 
                ctx.program_id
            );
            
            let seeds = &[POOL_SEED.as_bytes(), &a_pool.auction_id.to_le_bytes(), a_pool.mint.as_ref(), &[bump]];
            let signer = &[&seeds[..]];
    
            if a_pool.mint.to_string() == SOL_KEY.to_string() {
                transfer_lamports_from_treasury(
                    &ctx.accounts.treasury,
                    &ctx.accounts.bidder,
                    ctx.program_id,
                    price
                )?;
            } else {
                token::transfer(
                    ctx.accounts.transfer_context().with_signer(signer), 
                    price
                )?;
            }
    
        }
      
        Ok(())
    }

    pub fn set_winner(ctx: Context<SetWinnerContext>) -> Result<()> {
        {
            let mut a_pool = ctx.accounts.pool.load_mut()?;
            let current_time = get_current_time()?;
            require!(
                current_time >= a_pool.end_time, 
                AuctionError::NotFinishAuction
            );
    
            require!(a_pool.state == 0, AuctionError::AlreadySetWinner);
            
            a_pool.set_winner()?;
            
            a_pool.state = 1;
        }

        Ok(())
    }
}
