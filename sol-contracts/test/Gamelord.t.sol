// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Gamelord} from "../src/Gamelord.sol";

contract GamelordTest is Test {
    Gamelord public gamelord;

    function setUp() public {
        gamelord = new Gamelord();
        gamelord.setNumber(0);
    }

    function test_Increment() public {
        gamelord.increment();
        assertEq(gamelord.number(), 1);
    }

    function testFuzz_SetNumber(uint256 x) public {
        gamelord.setNumber(x);
        assertEq(gamelord.number(), x);
    }
}
