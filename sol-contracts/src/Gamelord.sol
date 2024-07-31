// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;
import "forge-std/console.sol";

error Gamelord__NotOwner();

contract Gamelord {
    uint256 public constant MINIMUM_ETH = 0.0001 ether;
    enum Team {
        Team1,
        Team2
    }
    struct PlayerInfo {
        uint256 amountWagered;
        Team team;
    }
    mapping(address => PlayerInfo) private s_addressToPlayerInfo;
    address[] private s_players;

    function wager(Team team) public payable {
        require(msg.value >= MINIMUM_ETH, "You need to spend more ETH!");
        s_addressToPlayerInfo[msg.sender].amountWagered += msg.value;
        s_addressToPlayerInfo[msg.sender].team = team;
        s_players.push(msg.sender);
    }

    function getPlayerInfo(
        address fundingAddress
    ) public view returns (PlayerInfo memory) {
        return s_addressToPlayerInfo[fundingAddress];
    }

    function getPlayers() public view returns (address[] memory) {
        return s_players;
    }

    function releaseFunds(Team winningTeam) public {
        // find number of players in team
        uint256 winningTeamPlayerCount = 0;
        for (uint256 i = 0; i < s_players.length; i++) {
            address player = s_players[i];
            if (s_addressToPlayerInfo[player].team == winningTeam) {
                winningTeamPlayerCount++;
            }
        }

        // calc amount to be sent to each winning player
        uint256 amountPerPlayer = address(this).balance /
            winningTeamPlayerCount;

        // send amount to winners
        for (uint256 i = 0; i < s_players.length; i++) {
            address player = s_players[i];
            if (s_addressToPlayerInfo[player].team == winningTeam) {
                console.log("player %s", player);
                console.log("balance %s", address(this).balance);
                console.log("amountPerPlayer %s", amountPerPlayer);
                (bool success, bytes memory data) = player.call{
                    value: amountPerPlayer
                }("");
                require(
                    success,
                    string(
                        abi.encodePacked(
                            "Failed to send Eth to ",
                            toAsciiString(player),
                            ": ",
                            data
                        )
                    )
                );
            }
        }

        // reset all data
        for (uint256 i = 0; i < s_players.length; i++) {
            delete s_addressToPlayerInfo[s_players[i]];
        }
        delete s_players;
    }

    function toAsciiString(address x) internal pure returns (string memory) {
        bytes memory s = new bytes(40);
        for (uint i = 0; i < 20; i++) {
            bytes1 b = bytes1(uint8(uint(uint160(x)) / (2 ** (8 * (19 - i)))));
            bytes1 hi = bytes1(uint8(b) / 16);
            bytes1 lo = bytes1(uint8(b) - 16 * uint8(hi));
            s[2 * i] = char(hi);
            s[2 * i + 1] = char(lo);
        }
        return string(s);
    }

    function char(bytes1 b) internal pure returns (bytes1 c) {
        if (uint8(b) < 10) return bytes1(uint8(b) + 0x30);
        else return bytes1(uint8(b) + 0x57);
    }

    // TODO perms (who can release the funds)
    // address private immutable i_owner;
    // constructor() {
    //     i_owner = msg.sender;
    // }
    // modifier onlyOwner() {
    //     // require(msg.sender == i_owner);
    //     if (msg.sender != i_owner) revert Gamelord__NotOwner();
    //     _;
    // }
    // function getOwner() public view returns (address) {
    //     return i_owner;
    // }
}
