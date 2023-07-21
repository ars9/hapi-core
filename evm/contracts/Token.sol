// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract Token is ERC20 {
    constructor() ERC20("HAPI", "HAPI Test Token") {
        _mint(msg.sender, 100000000000000000000000);
    }
}
