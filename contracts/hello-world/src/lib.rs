#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, Address, Env, String, Symbol,
};

// NFT structure to store NFT details
#[contracttype]
#[derive(Clone)]
pub struct NFT {
    pub nft_id: u64,
    pub owner: Address,
    pub title: String,
    pub price: u64,
    pub is_listed: bool,
}

// Mapping NFT ID to NFT details
#[contracttype]
pub enum NFTBook {
    NFT(u64),
}

// Counter for generating unique NFT IDs
const NFT_COUNT: Symbol = symbol_short!("NFT_CNT");

// Marketplace statistics
#[contracttype]
#[derive(Clone)]
pub struct MarketStats {
    pub total_nfts: u64,
    pub listed_nfts: u64,
    pub sold_nfts: u64,
}

const MARKET_STATS: Symbol = symbol_short!("MKT_STAT");

#[contract]
pub struct NFTMarketplace;

#[contractimpl]
impl NFTMarketplace {
    // Function 1: Mint a new NFT
    pub fn mint_nft(env: Env, owner: Address, title: String, price: u64) -> u64 {
        owner.require_auth();

        let mut nft_count: u64 = env.storage().instance().get(&NFT_COUNT).unwrap_or(0);
        nft_count += 1;

        let new_nft = NFT {
            nft_id: nft_count,
            owner: owner.clone(),
            title: title.clone(),
            price,
            is_listed: false,
        };

        let mut stats = Self::get_market_stats(env.clone());
        stats.total_nfts += 1;

        env.storage()
            .instance()
            .set(&NFTBook::NFT(nft_count), &new_nft);
        env.storage().instance().set(&NFT_COUNT, &nft_count);
        env.storage().instance().set(&MARKET_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "NFT Minted with ID: {}", nft_count);
        nft_count
    }

    // Function 2: List NFT for sale
    pub fn list_nft(env: Env, seller: Address, nft_id: u64, price: u64) {
        seller.require_auth();

        let mut nft = Self::view_nft(env.clone(), nft_id);

        if nft.nft_id == 0 {
            log!(&env, "NFT does not exist");
            panic!("NFT does not exist");
        }

        if nft.owner != seller {
            log!(&env, "You are not the owner of this NFT");
            panic!("Not the owner");
        }

        if nft.is_listed {
            log!(&env, "NFT is already listed");
            panic!("Already listed");
        }

        nft.is_listed = true;
        nft.price = price;

        let mut stats = Self::get_market_stats(env.clone());
        stats.listed_nfts += 1;

        env.storage().instance().set(&NFTBook::NFT(nft_id), &nft);
        env.storage().instance().set(&MARKET_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(
            &env,
            "NFT ID: {} listed for sale at price: {}",
            nft_id,
            price
        );
    }

    // Function 3: Buy NFT
    pub fn buy_nft(env: Env, buyer: Address, nft_id: u64) {
        buyer.require_auth();

        let mut nft = Self::view_nft(env.clone(), nft_id);

        if nft.nft_id == 0 {
            log!(&env, "NFT does not exist");
            panic!("NFT does not exist");
        }

        if !nft.is_listed {
            log!(&env, "NFT is not listed for sale");
            panic!("Not listed");
        }

        if nft.owner == buyer {
            log!(&env, "You already own this NFT");
            panic!("Already owner");
        }

        let previous_owner = nft.owner.clone();
        nft.owner = buyer.clone();
        nft.is_listed = false;

        let mut stats = Self::get_market_stats(env.clone());
        stats.listed_nfts -= 1;
        stats.sold_nfts += 1;

        env.storage().instance().set(&NFTBook::NFT(nft_id), &nft);
        env.storage().instance().set(&MARKET_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);

        log!(&env, "NFT ID: {} sold to new owner", nft_id);
    }

    // Function 4: View NFT details
    pub fn view_nft(env: Env, nft_id: u64) -> NFT {
        let key = NFTBook::NFT(nft_id);

        env.storage().instance().get(&key).unwrap_or(NFT {
            nft_id: 0,
            owner: Address::from_string(&String::from_str(&env, "default")),
            title: String::from_str(&env, "Not Found"),
            price: 0,
            is_listed: false,
        })
    }

    // Helper function: Get marketplace statistics
    pub fn get_market_stats(env: Env) -> MarketStats {
        env.storage()
            .instance()
            .get(&MARKET_STATS)
            .unwrap_or(MarketStats {
                total_nfts: 0,
                listed_nfts: 0,
                sold_nfts: 0,
            })
    }
}
