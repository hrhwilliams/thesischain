use ed25519_dalek::VerifyingKey;

use crate::crypto;
use crate::error::ChainError;
use crate::state::KeyDirectory;
use crate::types::Block;

/// The blockchain: an ordered list of validated blocks and the derived key directory state.
pub struct Chain {
    blocks: Vec<Block>,
    state: KeyDirectory,
    /// When set, `RegisterDevice` transactions must include a valid
    /// identity attestation signed by this backend key.
    backend_key: Option<VerifyingKey>,
}

impl Chain {
    /// Create a new chain from a genesis block.
    ///
    /// The genesis block (index 0) is special:
    /// - `previous_hash` must be all zeros
    /// - The author is not checked against the authority set (there are none yet)
    /// - Its transactions bootstrap the initial authority set
    ///
    /// Pass `backend_key` to enable dual-authority attestation verification.
    /// Pass `None` to skip attestation checks (for unit tests or standalone mode).
    pub fn new(genesis: Block, backend_key: Option<VerifyingKey>) -> Result<Self, ChainError> {
        if genesis.header.index != 0 {
            return Err(ChainError::InvalidBlockIndex {
                expected: 0,
                got: genesis.header.index,
            });
        }

        if genesis.header.previous_hash != [0u8; 32] {
            return Err(ChainError::InvalidPreviousHash);
        }

        // Verify the block's cryptographic integrity
        crypto::verify_block(&genesis)?;

        // Verify each transaction signature
        for tx in &genesis.transactions {
            crypto::verify_transaction(tx)?;
        }

        // Apply transactions to build initial state
        // Genesis transactions skip attestation checks (trusted setup)
        let mut state = KeyDirectory::default();
        for tx in &genesis.transactions {
            state.apply_transaction(tx, 0, None)?;
        }

        Ok(Self {
            blocks: vec![genesis],
            state,
            backend_key,
        })
    }

    /// Validate and append a block to the chain.
    pub fn append(&mut self, block: Block) -> Result<(), ChainError> {
        self.validate_block(&block)?;

        // Apply all transactions to state
        for tx in &block.transactions {
            self.state
                .apply_transaction(tx, block.header.index, self.backend_key.as_ref())?;
        }

        self.blocks.push(block);
        Ok(())
    }

    /// Validate a block against the current chain state without appending it.
    pub fn validate_block(&self, block: &Block) -> Result<(), ChainError> {
        let expected_index = self.height();
        if block.header.index != expected_index {
            return Err(ChainError::InvalidBlockIndex {
                expected: expected_index,
                got: block.header.index,
            });
        }

        // Check block linkage
        let last_block = self
            .blocks
            .last()
            .expect("chain always has at least the genesis block");
        let expected_hash = crypto::hash_block(last_block)?;
        if block.header.previous_hash != expected_hash {
            return Err(ChainError::InvalidPreviousHash);
        }

        // Check timestamp ordering
        if block.header.timestamp < last_block.header.timestamp {
            return Err(ChainError::InvalidTimestamp);
        }

        // Check the author is an authority
        if !self.state.is_authority(&block.header.author) {
            return Err(ChainError::UnauthorizedBlockAuthor);
        }

        // Verify cryptographic integrity (signature + transactions hash)
        crypto::verify_block(block)?;

        // Verify each transaction signature
        for tx in &block.transactions {
            crypto::verify_transaction(tx)?;
        }

        Ok(())
    }

    /// The number of blocks in the chain (next expected block index).
    #[must_use]
    pub const fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// SHA-256 hash of the latest block.
    pub fn head_hash(&self) -> Result<[u8; 32], ChainError> {
        let last = self
            .blocks
            .last()
            .expect("chain always has at least the genesis block");
        crypto::hash_block(last)
    }

    /// Access the derived key directory state.
    #[must_use]
    pub const fn state(&self) -> &KeyDirectory {
        &self.state
    }

    /// Get a block by index.
    #[must_use]
    pub fn get_block(&self, index: u64) -> Option<&Block> {
        #[allow(clippy::cast_possible_truncation)]
        self.blocks.get(index as usize)
    }

    /// Get all blocks from a given index onwards (for syncing peers).
    #[must_use]
    pub fn blocks_from(&self, from_index: u64) -> &[Block] {
        #[allow(clippy::cast_possible_truncation)]
        let start = from_index as usize;
        if start >= self.blocks.len() {
            return &[];
        }
        &self.blocks[start..]
    }
}
