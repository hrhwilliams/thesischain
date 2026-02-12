use ed25519_dalek::SigningKey;
use rand::RngCore;
use uuid::Uuid;

use crate::chain::Chain;
use crate::crypto;
use crate::error::ChainError;
use crate::genesis::{GenesisDevice, create_genesis};
use crate::types::{IdentityAttestation, Transaction};

fn random_signing_key() -> SigningKey {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    SigningKey::from_bytes(&bytes)
}

fn random_x25519() -> [u8; 32] {
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Create a dummy attestation for tests that don't verify attestations.
fn dummy_attestation(user_id: Uuid, device_id: Uuid) -> IdentityAttestation {
    IdentityAttestation {
        user_id,
        device_id,
        issued_at: 0,
        backend_key: [0u8; 32],
        signature: ed25519_dalek::Signature::from_bytes(&[0u8; 64]),
    }
}

fn make_genesis(key: &SigningKey) -> crate::Block {
    let device = GenesisDevice {
        user_id: Uuid::now_v7(),
        device_id: Uuid::now_v7(),
        signing_key: key.clone(),
        x25519: random_x25519(),
    };
    create_genesis(key, 1000, &[device], None).expect("genesis creation failed")
}

// --- Genesis tests -------------------------------------------------------

#[test]
fn genesis_block_creates_valid_chain() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let chain = Chain::new(genesis, None).expect("chain creation failed");

    assert_eq!(chain.height(), 1);
    assert!(chain.state().is_authority(&key.verifying_key().to_bytes()));
}

#[test]
fn genesis_with_multiple_devices() {
    let key1 = random_signing_key();
    let key2 = random_signing_key();

    let devices = vec![
        GenesisDevice {
            user_id: Uuid::now_v7(),
            device_id: Uuid::now_v7(),
            signing_key: key1.clone(),
            x25519: random_x25519(),
        },
        GenesisDevice {
            user_id: Uuid::now_v7(),
            device_id: Uuid::now_v7(),
            signing_key: key2.clone(),
            x25519: random_x25519(),
        },
    ];

    let genesis = create_genesis(&key1, 1000, &devices, None).expect("genesis creation failed");
    let chain = Chain::new(genesis, None).expect("chain creation failed");

    assert!(chain.state().is_authority(&key1.verifying_key().to_bytes()));
    assert!(chain.state().is_authority(&key2.verifying_key().to_bytes()));
}

// --- Transaction signing/verification ------------------------------------

#[test]
fn sign_and_verify_transaction() {
    let key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();
    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };

    let signed = crypto::sign_transaction(tx, 0, &key).expect("signing failed");
    crypto::verify_transaction(&signed).expect("verification failed");
}

#[test]
fn tampered_transaction_fails_verification() {
    let key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();
    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };

    let mut signed = crypto::sign_transaction(tx, 0, &key).expect("signing failed");
    // Tamper with the nonce
    signed.nonce = 999;

    assert!(crypto::verify_transaction(&signed).is_err());
}

// --- Block append tests --------------------------------------------------

#[test]
fn append_valid_block() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let mut chain = Chain::new(genesis, None).expect("chain creation failed");

    let new_key = random_signing_key();
    let device_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: new_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };

    // Nonce 1 because the genesis already used nonce 0 for the bootstrap device
    let signed = crypto::sign_transaction(tx, 0, &new_key).expect("signing failed");

    let previous_hash = chain.head_hash().expect("tip hash failed");
    let block = crypto::sign_block(1, 2000, previous_hash, vec![signed], &key)
        .expect("block signing failed");

    chain.append(block).expect("append failed");
    assert_eq!(chain.height(), 2);

    // New device should be queryable
    let record = chain
        .state()
        .get_device(device_id)
        .expect("device not found");
    assert_eq!(record.ed25519, new_key.verifying_key().to_bytes());
}

#[test]
fn reject_block_with_wrong_index() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let mut chain = Chain::new(genesis, None).expect("chain creation failed");

    let previous_hash = chain.head_hash().expect("tip hash failed");
    // Wrong index: 5 instead of 1
    let block =
        crypto::sign_block(5, 2000, previous_hash, vec![], &key).expect("block signing failed");

    match chain.append(block) {
        Err(ChainError::InvalidBlockIndex {
            expected: 1,
            got: 5,
        }) => {}
        other => panic!("expected InvalidBlockIndex, got: {other:?}"),
    }
}

#[test]
fn reject_block_with_wrong_previous_hash() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let mut chain = Chain::new(genesis, None).expect("chain creation failed");

    // Use wrong previous hash
    let block =
        crypto::sign_block(1, 2000, [0xAB; 32], vec![], &key).expect("block signing failed");

    match chain.append(block) {
        Err(ChainError::InvalidPreviousHash) => {}
        other => panic!("expected InvalidPreviousHash, got: {other:?}"),
    }
}

#[test]
fn reject_block_from_unauthorized_author() {
    let authority_key = random_signing_key();
    let genesis = make_genesis(&authority_key);
    let mut chain = Chain::new(genesis, None).expect("chain creation failed");

    let rogue_key = random_signing_key();
    let previous_hash = chain.head_hash().expect("tip hash failed");
    let block = crypto::sign_block(1, 2000, previous_hash, vec![], &rogue_key)
        .expect("block signing failed");

    match chain.append(block) {
        Err(ChainError::UnauthorizedBlockAuthor) => {}
        other => panic!("expected UnauthorizedBlockAuthor, got: {other:?}"),
    }
}

#[test]
fn reject_block_with_bad_timestamp() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let mut chain = Chain::new(genesis, None).expect("chain creation failed");

    let previous_hash = chain.head_hash().expect("tip hash failed");
    // Timestamp before genesis (1000)
    let block =
        crypto::sign_block(1, 500, previous_hash, vec![], &key).expect("block signing failed");

    match chain.append(block) {
        Err(ChainError::InvalidTimestamp) => {}
        other => panic!("expected InvalidTimestamp, got: {other:?}"),
    }
}

// --- State tests ---------------------------------------------------------

#[test]
fn get_user_devices_returns_all_devices() {
    let key1 = random_signing_key();
    let key2 = random_signing_key();
    let user_id = Uuid::now_v7();
    let device1_id = Uuid::now_v7();
    let device2_id = Uuid::now_v7();

    let devices = vec![
        GenesisDevice {
            user_id,
            device_id: device1_id,
            signing_key: key1.clone(),
            x25519: random_x25519(),
        },
        GenesisDevice {
            user_id,
            device_id: device2_id,
            signing_key: key2.clone(),
            x25519: random_x25519(),
        },
    ];

    let genesis = create_genesis(&key1, 1000, &devices, None).expect("genesis failed");
    let chain = Chain::new(genesis, None).expect("chain failed");

    let user_devices = chain.state().get_user_devices(user_id);
    assert_eq!(user_devices.len(), 2);
}

#[test]
fn duplicate_device_id_rejected() {
    let key1 = random_signing_key();
    let device_id = Uuid::now_v7();

    let genesis = make_genesis(&key1);
    let mut chain = Chain::new(genesis, None).expect("chain failed");

    // Register a device
    let key2 = random_signing_key();
    let user_id = Uuid::now_v7();
    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: key2.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };
    let signed = crypto::sign_transaction(tx, 0, &key2).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block =
        crypto::sign_block(1, 2000, previous_hash, vec![signed], &key1).expect("block sign failed");
    chain.append(block).expect("append failed");

    // Try to register the same device_id again
    let key3 = random_signing_key();
    let user_id2 = Uuid::now_v7();
    let tx2 = Transaction::RegisterDevice {
        user_id: user_id2,
        device_id, // same ID!
        ed25519: key3.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id2, device_id),
    };
    let signed2 = crypto::sign_transaction(tx2, 0, &key3).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block2 = crypto::sign_block(2, 3000, previous_hash, vec![signed2], &key1)
        .expect("block sign failed");

    match chain.append(block2) {
        Err(ChainError::DuplicateDeviceId(id)) if id == device_id => {}
        other => panic!("expected DuplicateDeviceId, got: {other:?}"),
    }
}

// --- Nonce replay rejection ----------------------------------------------

#[test]
fn nonce_replay_rejected() {
    let authority = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, None).expect("chain failed");

    let key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();
    let tx1 = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };
    let signed1 = crypto::sign_transaction(tx1, 0, &key).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block1 = crypto::sign_block(1, 2000, previous_hash, vec![signed1], &authority)
        .expect("block sign failed");
    chain.append(block1).expect("append failed");

    // Try another tx with the same nonce (0) from the same signer
    let user_id2 = Uuid::now_v7();
    let device_id2 = Uuid::now_v7();
    let tx2 = Transaction::RegisterDevice {
        user_id: user_id2,
        device_id: device_id2,
        ed25519: key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id2, device_id2),
    };
    let signed2 = crypto::sign_transaction(tx2, 0, &key).expect("sign failed"); // nonce 0 again!

    let previous_hash = chain.head_hash().expect("tip hash");
    let block2 = crypto::sign_block(2, 3000, previous_hash, vec![signed2], &authority)
        .expect("block sign failed");

    match chain.append(block2) {
        Err(ChainError::InvalidNonce { .. }) => {}
        other => panic!("expected InvalidNonce, got: {other:?}"),
    }
}

// --- Device revocation ---------------------------------------------------

#[test]
fn revoked_device_loses_authority() {
    let authority = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, None).expect("chain failed");

    // Register a second device
    let device_key = random_signing_key();
    let device_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let register_tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: device_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };
    let signed_reg = crypto::sign_transaction(register_tx, 0, &device_key).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block1 = crypto::sign_block(1, 2000, previous_hash, vec![signed_reg], &authority)
        .expect("block sign failed");
    chain.append(block1).expect("append failed");

    assert!(
        chain
            .state()
            .is_authority(&device_key.verifying_key().to_bytes())
    );

    // Revoke the device
    let revoke_tx = Transaction::RevokeDevice { device_id };
    let signed_revoke = crypto::sign_transaction(revoke_tx, 1, &device_key).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block2 = crypto::sign_block(2, 3000, previous_hash, vec![signed_revoke], &authority)
        .expect("block sign failed");
    chain.append(block2).expect("append failed");

    // Device should no longer be an authority
    assert!(
        !chain
            .state()
            .is_authority(&device_key.verifying_key().to_bytes())
    );

    // Device record should be marked as revoked
    let record = chain
        .state()
        .get_device(device_id)
        .expect("device not found");
    assert!(record.revoked);
}

// --- Key update ----------------------------------------------------------

#[test]
fn update_device_keys() {
    let authority = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, None).expect("chain failed");

    // Register a device
    let old_key = random_signing_key();
    let device_id = Uuid::now_v7();
    let user_id = Uuid::now_v7();
    let register_tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: old_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation: dummy_attestation(user_id, device_id),
    };
    let signed_reg = crypto::sign_transaction(register_tx, 0, &old_key).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block1 = crypto::sign_block(1, 2000, previous_hash, vec![signed_reg], &authority)
        .expect("block sign failed");
    chain.append(block1).expect("append failed");

    // Update to new keys
    let new_key = random_signing_key();
    let new_x25519 = random_x25519();
    let update_tx = Transaction::UpdateDeviceKeys {
        device_id,
        new_ed25519: new_key.verifying_key().to_bytes(),
        new_x25519,
    };
    let signed_update = crypto::sign_transaction(update_tx, 1, &old_key).expect("sign failed");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block2 = crypto::sign_block(2, 3000, previous_hash, vec![signed_update], &authority)
        .expect("block sign failed");
    chain.append(block2).expect("append failed");

    // Check that keys were updated
    let record = chain
        .state()
        .get_device(device_id)
        .expect("device not found");
    assert_eq!(record.ed25519, new_key.verifying_key().to_bytes());
    assert_eq!(record.x25519, new_x25519);

    // Old key should no longer be authority, new key should be
    assert!(
        !chain
            .state()
            .is_authority(&old_key.verifying_key().to_bytes())
    );
    assert!(
        chain
            .state()
            .is_authority(&new_key.verifying_key().to_bytes())
    );
}

// --- Block sync helpers --------------------------------------------------

#[test]
fn blocks_from_returns_correct_slice() {
    let key = random_signing_key();
    let genesis = make_genesis(&key);
    let mut chain = Chain::new(genesis, None).expect("chain failed");

    // Append a block
    let previous_hash = chain.head_hash().expect("tip hash");
    let block1 =
        crypto::sign_block(1, 2000, previous_hash, vec![], &key).expect("block sign failed");
    chain.append(block1).expect("append failed");

    assert_eq!(chain.blocks_from(0).len(), 2);
    assert_eq!(chain.blocks_from(1).len(), 1);
    assert_eq!(chain.blocks_from(2).len(), 0);
    assert_eq!(chain.blocks_from(100).len(), 0);
}

// --- Attestation tests ---------------------------------------------------

#[test]
fn attestation_sign_and_verify() {
    let backend_key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();

    let attestation =
        crypto::sign_attestation(user_id, device_id, 1000, &backend_key).expect("sign failed");
    crypto::verify_attestation(&attestation, &backend_key.verifying_key())
        .expect("verification failed");
}

#[test]
fn attestation_with_wrong_key_rejected() {
    let backend_key = random_signing_key();
    let wrong_key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();

    let attestation =
        crypto::sign_attestation(user_id, device_id, 1000, &backend_key).expect("sign failed");

    match crypto::verify_attestation(&attestation, &wrong_key.verifying_key()) {
        Err(ChainError::InvalidAttestation(_)) => {}
        other => panic!("expected InvalidAttestation, got: {other:?}"),
    }
}

#[test]
fn register_device_with_valid_attestation() {
    let authority = random_signing_key();
    let backend_key = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, Some(backend_key.verifying_key())).expect("chain failed");

    let device_key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();

    let attestation =
        crypto::sign_attestation(user_id, device_id, 2000, &backend_key).expect("sign attest");

    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: device_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation,
    };
    let signed = crypto::sign_transaction(tx, 0, &device_key).expect("sign tx");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block = crypto::sign_block(1, 2000, previous_hash, vec![signed], &authority)
        .expect("sign block");
    chain.append(block).expect("append should succeed with valid attestation");

    assert!(chain.state().get_device(device_id).is_some());
}

#[test]
fn register_device_with_invalid_attestation_rejected() {
    let authority = random_signing_key();
    let backend_key = random_signing_key();
    let wrong_key = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, Some(backend_key.verifying_key())).expect("chain failed");

    let device_key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();

    // Sign attestation with wrong key
    let attestation =
        crypto::sign_attestation(user_id, device_id, 2000, &wrong_key).expect("sign attest");

    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: device_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation,
    };
    let signed = crypto::sign_transaction(tx, 0, &device_key).expect("sign tx");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block = crypto::sign_block(1, 2000, previous_hash, vec![signed], &authority)
        .expect("sign block");

    match chain.append(block) {
        Err(ChainError::InvalidAttestation(_)) => {}
        other => panic!("expected InvalidAttestation, got: {other:?}"),
    }
}

#[test]
fn register_device_attestation_field_mismatch_rejected() {
    let authority = random_signing_key();
    let backend_key = random_signing_key();
    let genesis = make_genesis(&authority);
    let mut chain = Chain::new(genesis, Some(backend_key.verifying_key())).expect("chain failed");

    let device_key = random_signing_key();
    let user_id = Uuid::now_v7();
    let device_id = Uuid::now_v7();
    let wrong_user_id = Uuid::now_v7();

    // Attestation for a different user_id than the transaction
    let attestation =
        crypto::sign_attestation(wrong_user_id, device_id, 2000, &backend_key).expect("sign");

    let tx = Transaction::RegisterDevice {
        user_id,
        device_id,
        ed25519: device_key.verifying_key().to_bytes(),
        x25519: random_x25519(),
        attestation,
    };
    let signed = crypto::sign_transaction(tx, 0, &device_key).expect("sign tx");

    let previous_hash = chain.head_hash().expect("tip hash");
    let block = crypto::sign_block(1, 2000, previous_hash, vec![signed], &authority)
        .expect("sign block");

    match chain.append(block) {
        Err(ChainError::InvalidAttestation(msg)) => {
            assert!(msg.contains("do not match"), "unexpected message: {msg}");
        }
        other => panic!("expected InvalidAttestation, got: {other:?}"),
    }
}
