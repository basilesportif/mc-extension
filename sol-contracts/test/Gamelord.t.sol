// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Gamelord} from "../src/Gamelord.sol";

contract GamelordTest is Test {
    Gamelord public gamelord;

    address USER = makeAddr("user");
    address USER2 = makeAddr("user2");
    uint256 constant SEND_VALUE = 0.1 ether;
    uint256 constant STARTING_BALANCE = 10 ether;
    uint256 constant GAS_PRICE = 1;

    function setUp() public {
        gamelord = new Gamelord();
        vm.deal(USER, STARTING_BALANCE);
        vm.deal(USER2, STARTING_BALANCE);
    }

    function testMinimumEthIs005() public view {
        assertEq(gamelord.MINIMUM_ETH(), 0.05 ether);
    }

    function testWagerFailsWithoutEnoughEth() public {
        vm.expectRevert(); // means that the next line should revert for the test to pass
        gamelord.wager(Gamelord.Team.Team1); // send 0 eth, meaning tx will revert, and test will succeed
    }

    // wager amount is correct
    // wager team is correct
    // get player info works correctly
    function test_Wager() public {
        vm.prank(USER);
        gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team1);
        Gamelord.PlayerInfo memory playerInfo = gamelord.getPlayerInfo(USER);
        assertEq(uint256(playerInfo.team), uint256(Gamelord.Team.Team1));
        assertEq(playerInfo.amountWagered, SEND_VALUE);
    }

    function test_getPlayers() public {
        vm.prank(USER);
        gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team1);
        vm.prank(USER2);
        gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team2);
        address[] memory players = gamelord.getPlayers();
        assertEq(players.length, 2);
    }

    function test_releaseFunds() public {
        uint256 startingContractBalance = address(gamelord).balance;
        assertEq(startingContractBalance, 0);

        vm.prank(USER);
        gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team1);
        vm.prank(USER2);
        gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team2);

        assertEq(2 * SEND_VALUE, address(gamelord).balance);

        assertEq(USER.balance, STARTING_BALANCE - SEND_VALUE);
        assertEq(USER2.balance, STARTING_BALANCE - SEND_VALUE);

        gamelord.releaseFunds(Gamelord.Team.Team1);

        assertEq(USER.balance, STARTING_BALANCE + SEND_VALUE);
        assertEq(USER2.balance, STARTING_BALANCE - SEND_VALUE);
    }

    function test_releaseFundsMultiplePlayers() public {
        uint256 startingContractBalance = address(gamelord).balance;
        assertEq(startingContractBalance, 0);

        uint160 numberOfPlayers = 10;
        for (uint160 i = 0; i < numberOfPlayers; i++) {
            //  hoax => vm.prank + vm.deal (generate acc with a balance)
            hoax(address(i*2), STARTING_BALANCE); // really weird bug if i use address(i)
                                                  // contract errors when sending eth to 0x0...09
            if (i % 2 == 0) {
                gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team1);
            } else {
                gamelord.wager{value: SEND_VALUE}(Gamelord.Team.Team2);
            }
        }

        uint256 newContractBalance = address(gamelord).balance;
        assertEq(newContractBalance, SEND_VALUE * numberOfPlayers);


        // testing with Team2 == winning team
        gamelord.releaseFunds(Gamelord.Team.Team2);
        assertEq(address(gamelord).balance, 0);

        for (uint160 i = 0; i < numberOfPlayers; i++) {
            if (i % 2 == 0) {
                // team1
                assertEq(address(i*2).balance, STARTING_BALANCE - SEND_VALUE);
            } else {
                // team2
                assertEq(address(i*2).balance, STARTING_BALANCE + SEND_VALUE);
            }
        }
    }

    // function testFuzz_Wager(uint256 amount) public {
    //     gamelord.wager(Gamelord.Team.Team1);
    // }
}
