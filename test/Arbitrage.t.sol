// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;
pragma abicoder v2;

import "../lib/forge-std/src/Test.sol";
import "../src/Arbitrage.sol";

contract ArbitrageTest is Test {
    Arbitrage public arbitrage;
    FakeERC20 public fakeERC20;
    EvilCoinbase public evilCoinbase;

    modifier TestCannotOperateByNotOwner() {
        vm.expectRevert(Unauthorized.selector);
        vm.prank(address(0));

        _;
    }

    function setUp() public {
        arbitrage = new Arbitrage();
        fakeERC20 = new FakeERC20("FakeERC20", "Fake", 18);
        evilCoinbase = new EvilCoinbase();
    }

    function testCannotFlashByNotOwner() public TestCannotOperateByNotOwner {
        arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(
            bytes32(""), uint256(0), [abi.encode(address(0), uint256(0), bytes(""))]
        ));
    }
    function testCannotFlashByUncleBlock() public {
        vm.expectRevert(UncleBlock.selector);
        vm.roll(3);
        vm.prank(address(this), address(this));

        bytes[] memory payloads = new bytes[](1);
        payloads[0] = abi.encode(address(0), uint256(0), bytes(""));
        arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(blockhash(block.number - 2), uint256(0), payloads));
    }
    function testCannotFlashBySelfCall() public {
        vm.expectRevert(SelfCall.selector);
        vm.prank(address(this), address(this));

        bytes[] memory payloads = new bytes[](1);
        payloads[0] = abi.encode(address(arbitrage), uint256(0), bytes(""));
        arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(bytes32(""), uint256(0), payloads));
    }
    function testCannotFlashBySufficientIncome() public {
        vm.expectRevert(SufficientIncome.selector);
        vm.prank(address(this), address(this));

        bytes[] memory payloads = new bytes[](1);
        payloads[0] = abi.encode(address(0), uint256(0), bytes(""));
        arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(bytes32(""), uint256(0), payloads));
    }
    function testCannotFlashByReentrancy() public {
        // ERC20 Asset
        fakeERC20.mint(address(arbitrage), 1 ether);
        evilCoinbase.setToken(address(fakeERC20));

        // Bribe Fund
        vm.deal(address(evilCoinbase), 1 wei);
        vm.deal(address(fakeERC20), 1 ether);
        vm.coinbase(address(evilCoinbase));
        vm.prank(address(this), address(this));
        vm.expectRevert(CoinbaseCall.selector);

        bytes[] memory payloads = new bytes[](1);
        payloads[0] = abi.encode(address(fakeERC20), uint256(0), abi.encodeWithSignature("exploit()", ""));
        arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(blockhash(block.number - 1), uint256(0), payloads));
        // assertEq(fakeERC20.balanceOf(address(evilCoinbase)), 1 ether);
    }
    function testFlashByOwner() public {
        vm.deal(address(arbitrage), 0 ether);
        vm.deal(address(fakeERC20), 2 ether);
        vm.prank(address(this), address(this));

        bytes[] memory payloads = new bytes[](2);
        payloads[0] = abi.encode(address(fakeERC20), uint256(0), abi.encodeWithSignature("exploit()", ""));
        payloads[1] = abi.encode(address(fakeERC20), uint256(0), abi.encodeWithSignature("exploit()", ""));
        assertEq(
            arbitrage.onFlashLoan(address(0), address(0), 0, 0, abi.encode(blockhash(block.number - 1), uint256(0), payloads)),
            keccak256("ERC3156FlashBorrower.onFlashLoan")
        );
        assertEq(address(arbitrage).balance, 2 ether);
    }

    function testWithdrawByOwner() public {
        vm.deal(address(this), 0 ether);
        vm.deal(address(arbitrage), 1 ether);

        arbitrage.withdraw();
        assertEq(address(this).balance, 1 ether);
    }
    function testCannotWithdrawByNotOwner() public TestCannotOperateByNotOwner {
        arbitrage.withdraw();
    }

    function testRecoverERC20ByOwner() public {
        uint256 _balance = fakeERC20.balanceOf(address(this));
        fakeERC20.transfer(address(arbitrage), fakeERC20.balanceOf(address(this)));
        assertEq(fakeERC20.balanceOf(address(this)), 0);
        arbitrage.recoverERC20(address(fakeERC20));
        assertEq(fakeERC20.balanceOf(address(this)), _balance);
    }
    function testCannotRecoverERC20ByNotOwner() public TestCannotOperateByNotOwner {
        arbitrage.recoverERC20(address(0));
    }

    fallback() external payable {}
}

contract FakeERC20 is ERC20, Owned {
    constructor(string memory _name, string memory _symbol, uint8 _decimals) ERC20(_name, _symbol, _decimals) Owned(msg.sender) {
        _mint(msg.sender, 1 ether);
    }

    function exploit() external {
        payable(msg.sender).transfer(1 ether);
    }

    function mint(address _to, uint256 _amount) public onlyOwner {
        _mint(_to, _amount);
    }
}

contract EvilCoinbase {
    uint256 private _once;
    address private _token;

    function setToken(address t) external {
        _token = t;
    }

    fallback() external payable {
        if (_once < block.number) {
            _once = block.number;
            // Only ERC tokens can be attacked
            IERC3156FlashBorrower(msg.sender).onFlashLoan(address(0), address(0), 0, 0, abi.encode(
                bytes32(""), uint256(0), [abi.encode(
                    address(_token), uint256(0), abi.encodeWithSignature(
                        "transfer(address,uint256)",
                        address(this), IERC20(_token).balanceOf(msg.sender)
                    )
                )]
            ));
        } else {
            payable(msg.sender).transfer(1 wei);
        }
    }
}
