// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "contract/Arbitrage.sol";

interface IUniswapV2Pair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
}

contract NotFlationTokenTest is Test {
    struct Token {
        address addr;
        address lp;
    }

    struct BalanceInfo {
        uint256 from;
        uint256 to;
        uint256 lp;
        uint256 base_reverse;
        uint256 pair_reverse;
    }

    mapping (address => bool) public notFlationTokenMap;

    function testETHNotFlationToken1() public {
        vm.createSelectFork("https://rpc.ankr.com/eth", 16428369);
        // 0xa806617cdd8ed760ed25cec61abf642f4889749c3cede45c46f27d60f0941bd1
        address QTN = 0xC9fa8F4CFd11559b50c5C7F6672B9eEa2757e1bd;
        // 0xd099a41830b964e93415e9a8607cd92567e40d3eeb491d52f3b66eee6b0357eb
        address UPStkn = 0xFFeE5EcDE135a7b10A0Ac0E6e617798e6aD3D0D6;
        address WETH = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
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

        // transfer self increment
        assert(notFlationTokenMap[UPStkn] == true);

        // weth is safety
        assert(notFlationTokenMap[WETH] == false);
    }

    function testETHNotFlationToken2() public {
        vm.createSelectFork("https://rpc.ankr.com/eth");
        address HIMEI = 0x81b6E6EE0Bd303A1f1Ef7D63f9A071F7eF2abe09;

        Token[1] memory TOKEN_LIST = [
            Token(HIMEI, 0x4a449fFD26332170b19450FB573864407385B2d4)
        ];

        for (uint i = 0;i < TOKEN_LIST.length; i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // slight lp reverse change are also detected, even is safety
        assert(notFlationTokenMap[HIMEI] == true);
    }

    function testBSCNotFlationToken() public {
        vm.createSelectFork("https://rpc.ankr.com/bsc", 24912666);
        // 0x5058c820fa0bb0daff2bd1b30151cf84c618dffe123546223b7089c8c2e18331
        address THOREUM = 0x79Fe086a4C03C5E38FF8074DEA9Ee0a18dC1AF4F;
        address WBNB = 0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c;
        Token[2] memory TOKEN_LIST = [
            Token(THOREUM, 0xd822E1737b1180F72368B2a9EB2de22805B67E34),
            Token(WBNB, 0x1CEa83EC5E48D9157fCAe27a19807BeF79195Ce1)
        ];

        for (uint16 i = 0;i < TOKEN_LIST.length;i++) {
            // ignore error
            address(this).call(abi.encodeWithSignature("check((address,address))", TOKEN_LIST[i]));
        }

        // not a simple transfer self increment, the specific arbitrage logic is not clear
        assert(notFlationTokenMap[THOREUM] == false);

        // wbnb is safety
        assert(notFlationTokenMap[WBNB] == false);
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
        // THOREUM exploit, transfer self with non-zero amount
        _transfer(_token, address(this), address(this), amount);
    }

    function _transfer(Token calldata _token, address from, address to, uint256 amount) internal {        
        BalanceInfo memory balance_before = _getBalanceInfo(_token, from, to);

        vm.prank(from);
        IERC20(_token.addr).transfer(to, amount);

        BalanceInfo memory balance_after = _getBalanceInfo(_token, from, to);

        if (
            (from != to && balance_after.from != balance_before.from - amount) ||
            (from == to && balance_after.from > balance_before.from) ||
            (to != _token.lp && balance_after.to > (from != to ? balance_before.to + amount : balance_before.to)) ||
            (balance_after.base_reverse * balance_before.pair_reverse != balance_after.pair_reverse * balance_before.base_reverse)
        ) {
            notFlationTokenMap[_token.addr] = true;
        }
    }

    function _getBalanceInfo(Token memory _token, address from, address to) view internal returns (BalanceInfo memory balanceInfo) {
        address token = _token.addr;
        address lp = _token.lp;
        bool swap = token == IUniswapV2Pair(lp).token1();
        (uint256 reverse0, uint256 reverse1,) = IUniswapV2Pair(lp).getReserves();

        balanceInfo = BalanceInfo(
            IERC20(token).balanceOf(from),
            IERC20(token).balanceOf(to),
            IERC20(token).balanceOf(lp),
            swap ? reverse1 : reverse0,
            swap ? reverse0 : reverse1
        );
    }
}
