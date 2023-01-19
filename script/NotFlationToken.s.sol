// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "contract//Arbitrage.sol";

contract NotFlationTokenScript is Test {
    // 0xa806617cdd8ed760ed25cec61abf642f4889749c3cede45c46f27d60f0941bd1
    address public constant QTN = 0xC9fa8F4CFd11559b50c5C7F6672B9eEa2757e1bd;
    // 0xd099a41830b964e93415e9a8607cd92567e40d3eeb491d52f3b66eee6b0357eb
    address public constant UPStkn = 0xFFeE5EcDE135a7b10A0Ac0E6e617798e6aD3D0D6;

    address public constant WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;

    struct Token {
        address addr;
        address lp;
    }

    mapping (address => bool) public notFlationTokenMap;

    function setUp() public {
        vm.createSelectFork("https://rpc.ankr.com/eth", 16428369);
    }

    function run() public {
        Token[3] memory TOKEN_LIST = [
            Token(QTN, 0xA8208dA95869060cfD40a23eb11F2158639c829B),
            Token(UPStkn, 0xa3f47DCFC09d9aaDb7Ac6ECAB257cf7283BFEe26),
            Token(WETH, 0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc)
        ];
        for (uint i = 0;i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // Unable to detect irregular contracts
        assert(notFlationTokenMap[QTN] == false);

        // lp pool
        assert(notFlationTokenMap[UPStkn] == true);

        // weth is safety
        assert(notFlationTokenMap[WETH] == false);
    }

    function check(Token calldata _token) external {
        address token = _token.addr;
        address lp = _token.lp;
        // get decimal
        uint8 decimal = IERC20(token).decimals();
        // set balance
        uint256 balance_this = 1 * (10 ** decimal);
        deal(token, address(this), balance_this);
        assert(IERC20(token).balanceOf(address(this)) == balance_this);
        
        // transfer from this to lp pool
        uint256 amount = balance_this / 2;
        _transfer(_token, address(this), address(lp), amount);
        // transfer from lp pool to this
        _transfer(_token, address(lp), address(this), amount);
        // UPStkn exploit, transfer self with zero amount
        _transfer(_token, address(this), address(this), 0);
    }

    function _transfer(Token calldata _token, address from, address to, uint256 amount) internal {
        address token = _token.addr;
        address lp = _token.lp;
        uint256 balance_from = IERC20(token).balanceOf(from);
        uint256 balance_to = IERC20(token).balanceOf(to);
        uint256 balance_lp = IERC20(token).balanceOf(lp);

        vm.prank(from);
        IERC20(token).transfer(to, amount);

        if (
            (IERC20(token).balanceOf(from) != balance_from - amount) ||
            (IERC20(token).balanceOf(to) != balance_to + amount) ||
            (from != lp && to != lp && IERC20(token).balanceOf(lp) != balance_lp)
        ) {
            notFlationTokenMap[token] = true;
            return;
        }
    }
}
