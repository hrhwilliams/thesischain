// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {KeyDirectory} from "../src/KeyDirectory.sol";

contract KeyDirectoryScript is Script {
    KeyDirectory public key_directory;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        key_directory = new KeyDirectory();

        vm.stopBroadcast();
    }
}
