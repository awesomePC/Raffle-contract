use anchor_lang::prelude::Pubkey;

pub const POOL_SEED: &str = "pool";


pub const MAX_BID_COUNT: usize = 100;
pub const MAX_AUCTION_ID_LEN: usize = 50;
pub const DECIMAL: u64 = 1000000000;
// ===================================================== mainnet ==================================================== //
pub const ADMIN_KEY: Pubkey = anchor_lang::solana_program::pubkey!("C8HXcXRqA6UjWAf1NTQXY7i4DMvMY9x3zbUhj9dyw2Yi");
pub const COMMUNITY_KEY: Pubkey = anchor_lang::solana_program::pubkey!("3XN3bRqf6Nnf8ZM9jjwf8sP1fT7oQ6m7XgpPfEUmJiob"); 
pub const PAY_TOKEN: Pubkey = anchor_lang::solana_program::pubkey!("9aeip1QTVXNUVbcQ14UMDssmxNv4ve7sg8cVyfHoeNmT");

pub const WITHDRAW_KEY: Pubkey = anchor_lang::solana_program::pubkey!("C8HXcXRqA6UjWAf1NTQXY7i4DMvMY9x3zbUhj9dyw2Yi");

pub const SOL_KEY: Pubkey = anchor_lang::solana_program::pubkey!("So11111111111111111111111111111111111111112");
// ===================================================== devnet ==================================================== //
// pub const ADMIN_KEY: Pubkey = anchor_lang::solana_program::pubkey!("C8HXcXRqA6UjWAf1NTQXY7i4DMvMY9x3zbUhj9dyw2Yi");
// pub const COMMUNITY_KEY: Pubkey = anchor_lang::solana_program::pubkey!("2XtHzHeZMAqGgdUztQTjbMGAyYo8SZSmveosuKRN25MQ"); 
// pub const PAY_TOKEN: Pubkey = anchor_lang::solana_program::pubkey!("55u5jMiJtwsvyo834R2mmcrxMGu7x2KvbrguJNbHFnEJ");
