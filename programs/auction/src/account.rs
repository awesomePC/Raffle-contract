use anchor_lang::prelude::*;

use crate::constants::*;
use crate::errors::*;


#[account(zero_copy)]
// #[repr(packed)]
pub struct User {
  pub admins: [Pubkey; 100],
  pub count: u32
}

impl Default for User {
  #[inline]
  fn default() -> User {
    User {
          admins: [anchor_lang::solana_program::pubkey!("3ttYrBAp5D2sTG2gaBjg8EtrZecqBQSBuFRhsqHWPYxX"); 100],
          count: 0
      }
  }
}

impl User {
  pub fn find_admin(&self, admin: Pubkey) -> usize {
    let mut index = 100;
    for i in 0..self.count as usize {
      if self.admins[i] == admin {
        index = i;
        break;
      }
    }

    index
  }

  pub fn add_admin(&mut self, admin: Pubkey) -> Result<bool> {
    self.admins[self.count as usize] =  admin;
    Ok(true)
  } 

  pub fn delete_admin(&mut self, admin: Pubkey) -> Result<bool> {
    let index = self.find_admin(admin);

    if index < 100 {
      for i in index..self.count as usize - 1 {
        self.admins[i] = self.admins[i + 1];
      }
    }

    return Ok(true);
  }
}

#[account(zero_copy)]
// #[repr(packed)]
pub struct Pool {
  pub auction_id: u64, 
  pub mint: Pubkey,
  pub start_time: u32,
  pub end_time: u32,
  pub min_price: u64,
  pub bid_increment: u64,
  pub bids: [Bid; MAX_BID_COUNT],
  pub count: u32,
  pub state: u32,
  pub min_nft_count: u32
}

impl Default for Pool {
  #[inline]
  fn default() -> Pool {
      Pool {
          auction_id: 0,
          mint: anchor_lang::solana_program::pubkey!("C8HXcXRqA6UjWAf1NTQXY7i4DMvMY9x3zbUhj9dyw2Yi"),
          start_time: 0,
          end_time: 0,
          min_price: 0,
          bid_increment: 0,
          bids: [
            Bid {
                ..Default::default()
            }; MAX_BID_COUNT
          ],
          count: 0,
          state: 0,
          min_nft_count: 0
      }
  }
}

impl Pool {

  fn find_bid(&self, bidder: Pubkey) -> usize {
    let mut index = MAX_BID_COUNT;
    for i in 0..self.count as usize {
      if self.bids[i].bidder == bidder {
        index = i;
        break;
      }
    }

    index
  }

  pub fn add_bid(&mut self, bidder: Pubkey, price: u64) -> Result<bool> {
    let index = self.find_bid(bidder);
    if index < MAX_BID_COUNT {
      return Ok(false);
    }
    self.bids[self.count as usize] = Bid {
      bidder,
      price,
      claimed: 0,
      is_winner: 0,
      // claimed_prize: 0
    };

    Ok(true)
  }

  pub fn update_bid(&mut self, bidder: Pubkey, price: u64) -> Result<u64> {
    let mut old_price: u64 = 0;
    let index = self.find_bid(bidder);

    if index < MAX_BID_COUNT {
      old_price = self.bids[index].price;
      self.bids[index].price = price;
    }

    Ok(old_price)
  }

  pub fn cancel_bid(&mut self, bidder: Pubkey) -> Result<u64> {
    let mut price: u64 = 0;
    let index = self.find_bid(bidder);

    if index < MAX_BID_COUNT {
      price = self.bids[index].price;
      for i in index..self.count as usize - 1 {
        self.bids[i] = self.bids[i + 1];
      }
    }

    return Ok(price);
  }
  
  pub fn exist_winner(&mut self) -> Result<bool> {
    let mut result = false;
    for i in 0..self.count as usize {
      if self.bids[i].is_winner == 1 {
        result = true;
        break;
      }
    }

    Ok(result)
  }

  pub fn set_winner(&mut self) -> Result<bool> {
    let mut winner_price: u64 = 0;
    let mut winner_index = 0;
    for i in 0..self.count as usize {
      if self.bids[i].price > 0 && self.bids[i].price > winner_price {
        // self.bids[i].is_winner = 1;
        winner_index = i;
        winner_price = self.bids[i].price;
      }
    }
    
    if winner_price > 0 {
      self.bids[winner_index].is_winner = 1;
    }

    Ok(winner_price != 0)
  }

  pub fn claim_bid(&mut self, bidder: Pubkey) -> Result<(u64, bool)> {
    let mut price: u64 = 0;
    let mut claimed: bool = false;

    let index = self.find_bid(bidder);

    if index < MAX_BID_COUNT {
      price = self.bids[index].price;
      if self.bids[index].claimed > 0 {
        claimed = true;
      }
      else {
        self.bids[index].claimed = 1;
      }
    }

    Ok((price, claimed))
  }

  pub fn get_bid_price(&mut self, bidder: Pubkey) -> Result<u64> {
    let mut price: u64 = 0;

    let index = self.find_bid(bidder);

    if index < MAX_BID_COUNT {
      price = self.bids[index].price;
    }

    Ok(price)
  }

  // pub fn claim_prize(&mut self, bidder: Pubkey) -> Result<bool> {
  //   let index = self.find_bid(bidder);
  //   let mut claimedPrize: bool = false;

  //   if index < MAX_BID_COUNT && self.bids[index].is_winner > 0{
  //     if self.bids[index].claimed_prize > 0 {
  //       claimedPrize = true;
  //     }
  //     else {
  //       self.bids[index].claimed_prize = 1;
  //       claimedPrize = true;
  //     }
  //   }

  //   Ok(claimedPrize)
  // }

  pub fn claim_prize(&mut self, bidder: Pubkey) -> Result<bool> {
    let index = self.find_bid(bidder);
    Ok(index < MAX_BID_COUNT && self.bids[index].is_winner > 0)
  }

}
#[zero_copy]
#[derive(Default, AnchorSerialize, AnchorDeserialize)]
pub struct Bid {
  pub bidder: Pubkey,
  pub price: u64,
  pub claimed: u32,
  pub is_winner: u32,
  // pub claimed_prize: u32
}


#[account]
pub struct AuctionState {
    pub create_fee: u64, // create fee
    pub claim_fee: u64, // claim fee
    pub ticket_fee: u64, // ticket fee
}