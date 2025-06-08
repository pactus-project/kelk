//! Mocking the blockchain for testing purpose

use super::{
    Blockchain,
    PARAM_ID_TRANSACTION_SIGNER,
    address::{ADDRESS_SIZE, Address},
};
use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::{any::Any, result::Result};
use kelk_env::{BlockchainAPI, HostError};
use rand::{Rng, SeedableRng, rngs::SmallRng};

/// mocks the blockchain for testing purpose.
pub struct MockBlockchain {
    map: BTreeMap<u32, Vec<u8>>,
    addr_gen_seed: u64,
}

impl MockBlockchain {
    /// instantiates a new blockchain mock
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            addr_gen_seed: 0,
        }
    }

    /// generates a random address for testing
    pub fn generate_new_address(&mut self) -> Address {
        self.addr_gen_seed += 1;
        let mut small_rng = SmallRng::seed_from_u64(self.addr_gen_seed);
        let mut buf = [0u8; ADDRESS_SIZE];
        small_rng.fill(&mut buf);
        Address::from_bytes(&buf).unwrap()
    }
    /// sets message sender for testing
    pub fn set_msg_sender(&mut self, addr: &Address) {
        self.map.insert(
            PARAM_ID_TRANSACTION_SIGNER,
            addr.clone().as_bytes().to_vec(),
        );
    }
}

impl Default for MockBlockchain {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockchainAPI for MockBlockchain {
    fn get_param<'a>(&self, param_id: u32) -> Result<Vec<u8>, HostError> {
        Ok(self.map.get(&param_id).unwrap().to_vec())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// mocks the blockchain for testing
pub fn mock_blockchain() -> Blockchain {
    Blockchain::new(Box::new(MockBlockchain::new()))
}
