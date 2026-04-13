// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {KeyDirectory} from "../src/KeyDirectory.sol";

contract KeyDirectoryTest is Test {
    KeyDirectory public key_directory;

    event DeviceAdded(bytes32 indexed user_hash, uint128 device_id, bytes32 x25519, bytes32 ed25519, uint256 timestamp);

    bytes32 constant USER_HASH = keccak256(abi.encodePacked("alice"));
    uint128 constant DEVICE_ID = 12345;
    bytes32 constant X25519_KEY = "x25519_dummy_key";
    bytes32 constant ED25519_KEY = "ed25519_dummy_key";

    uint128 constant DEVICE_ID_2 = 67890;
    bytes32 constant X25519_KEY_2 = "x25519_dummy_key_2";
    bytes32 constant ED25519_KEY_2 = "ed25519_dummy_key_2";

    function setUp() public {
        key_directory = new KeyDirectory();
    }

    function test_add_then_get_device() public {
        key_directory.add_first_device(USER_HASH, DEVICE_ID, X25519_KEY, ED25519_KEY);

        KeyDirectory.Device memory device = key_directory.get_device(USER_HASH, DEVICE_ID);

        assertEq(device.device_id, DEVICE_ID);
        assertEq(device.x25519, X25519_KEY);
        assertEq(device.ed25519, ED25519_KEY);
    }

    function test_get_all_devices() public {
        key_directory.add_first_device(USER_HASH, DEVICE_ID, X25519_KEY, ED25519_KEY);
        key_directory.add_device(USER_HASH, DEVICE_ID_2, X25519_KEY_2, ED25519_KEY_2, 1);

        KeyDirectory.Device[] memory devices = key_directory.get_all_devices(USER_HASH);

        assertEq(devices[0].device_id, DEVICE_ID);
        assertEq(devices[0].x25519, X25519_KEY);
        assertEq(devices[0].ed25519, ED25519_KEY);

        assertEq(devices[1].device_id, DEVICE_ID_2);
        assertEq(devices[1].x25519, X25519_KEY_2);
        assertEq(devices[1].ed25519, ED25519_KEY_2);
    }

    function test_get_all_devices_no_user() public view {
        KeyDirectory.Device[] memory devices = key_directory.get_all_devices("abcd");
        assertEq(devices.length, 0);
    }

    function test_get_nonce_no_user() public view {
        uint256 nonce = key_directory.get_nonce("abcd");
        assertEq(nonce, 0);
    }

    function test_add_device_emits_event() public {
        vm.expectEmit(true, false, false, true);
        emit DeviceAdded(USER_HASH, DEVICE_ID, X25519_KEY, ED25519_KEY, block.timestamp);
        key_directory.add_device(USER_HASH, DEVICE_ID, X25519_KEY, ED25519_KEY, 0);
    }

    function test_add_device_immutable() public {
        key_directory.add_first_device(USER_HASH, DEVICE_ID, X25519_KEY, ED25519_KEY);

        // DEVICE_ID is the same, which would overwrite the previous device
        vm.expectRevert("Device ID already exists");
        key_directory.add_device(USER_HASH, DEVICE_ID, X25519_KEY_2, ED25519_KEY_2, 1);
    }
}
