// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

contract KeyDirectory {
    address public relayer;

    constructor() {
        relayer = msg.sender;
    }

    struct Device {
        uint128 device_id;
        uint128 flags;
        bytes32 x25519;
        bytes32 ed25519;
    }

    mapping(bytes32 => Device[]) private devices;
    mapping(bytes32 => uint256) private nonces;

    event DeviceAdded(bytes32 indexed user_hash, uint128 device_id, bytes32 x25519, bytes32 ed25519, uint256 timestamp);

    function add_first_device(bytes32 user_hash, uint128 device_id, bytes32 x25519, bytes32 ed25519) public {
        require(msg.sender == relayer, "Unauthorized");
        require(devices[user_hash].length == 0, "Additional devices must be signed");

        devices[user_hash].push(Device({device_id: device_id, flags: 0, x25519: x25519, ed25519: ed25519}));

        nonces[user_hash] = 1;

        emit DeviceAdded(user_hash, device_id, x25519, ed25519, block.timestamp);
    }

    function add_device(bytes32 user_hash, uint128 device_id, bytes32 x25519, bytes32 ed25519, uint256 nonce) public {
        require(msg.sender == relayer, "Unauthorized");
        require(nonce == nonces[user_hash]);

        // Prevent operation if device already exists
        for (uint256 i = 0; i < devices[user_hash].length; i++) {
            require(devices[user_hash][i].device_id != device_id, "Device ID already exists");
        }

        devices[user_hash].push(Device({device_id: device_id, flags: 0, x25519: x25519, ed25519: ed25519}));

        nonces[user_hash] += 1;

        emit DeviceAdded(user_hash, device_id, x25519, ed25519, block.timestamp);
    }

    function get_device(bytes32 user_hash, uint128 device_id) public view returns (Device memory) {
        for (uint256 i = 0; i < devices[user_hash].length; i++) {
            if (devices[user_hash][i].device_id == device_id) {
                return devices[user_hash][i];
            }
        }

        revert("Device not found");
    }

    function get_all_devices(bytes32 user_hash) public view returns (Device[] memory) {
        return devices[user_hash];
    }

    function get_nonce(bytes32 user_hash) public view returns (uint256) {
        return nonces[user_hash];
    }
}
