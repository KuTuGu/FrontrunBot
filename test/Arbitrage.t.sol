// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/Arbitrage.sol";

contract ArbitrageTest is Test {
    Arbitrage public arbitrage;
    FakeERC20 public fakeERC20;
    FlashLender public flashLender;

    modifier TestCannotOperateByNotOwner() {
        vm.expectRevert(Unauthorized.selector);
        vm.prank(address(0));

        _;
    }

    function setUp() public {
        arbitrage = new Arbitrage();
        fakeERC20 = new FakeERC20("FakeERC20", "Fake", 18);
        flashLender = new FlashLender();

        vm.deal(address(flashLender), 1000 ether);
        vm.prank(address(flashLender));
        address(fakeERC20).call{value: 1000 ether}(abi.encodeWithSignature("deposit()", ""));
        arbitrage.setFlashLender(address(flashLender));
    }

    function testCannotArbitrageByNotOwner() public TestCannotOperateByNotOwner {
        arbitrage.run(abi.encode(bytes32(""), 0, bytes("")));
    }
    function testCannotArbitrageByUncleBlock() public {
        vm.expectRevert(UncleBlock.selector);
        vm.roll(3);
        arbitrage.run(abi.encode(blockhash(block.number - 2), 0, bytes("")));
    }
    function testCannotArbitrageBySufficientIncome() public {
        vm.expectRevert(SufficientIncome.selector);
        arbitrage.run(abi.encode(bytes32(""), 0, bytes("")));
    }
    function testFlashArbitrageByOwner() public {
        uint256 _lenderAmount = 1000 ether;

        bytes[] memory payloads = new bytes[](3);
        payloads[0] = abi.encode(address(fakeERC20), 0, abi.encodeWithSignature(
            "transferFrom(address,address,uint256)",
            address(flashLender), address(arbitrage), _lenderAmount
        ));
        payloads[1] = abi.encode(address(fakeERC20), 0, abi.encodeWithSignature("exploit()", ""));
        payloads[2] = abi.encode(address(fakeERC20), 0, abi.encodeWithSignature("withdraw(uint256)", 1 ether));

        assertEq(address(arbitrage).balance, 0 ether);
        assertEq(fakeERC20.balanceOf(address(arbitrage)), 0 ether);
        flashLender.flashLoan(
            arbitrage, address(fakeERC20), _lenderAmount,
            abi.encode(blockhash(block.number - 1), 0, payloads)
        );
        assertEq(address(arbitrage).balance, 1 ether);
        assertEq(fakeERC20.balanceOf(address(arbitrage)), 0 ether);
    }

    function testSetFlashLenderByOwner() public {
        arbitrage.setFlashLender(address(0));
    }
    function testCannotSetFlashLenderByNotOwner() public TestCannotOperateByNotOwner {
        arbitrage.setFlashLender(address(0));
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
    constructor(string memory _name, string memory _symbol, uint8 _decimals) ERC20(_name, _symbol, _decimals) Owned(msg.sender) {}

    function exploit() external {
        if (balanceOf[msg.sender] >= 1000 ether) {
            _mint(msg.sender, 2 ether);
        }
    }

    function deposit() payable external {
        _mint(msg.sender, msg.value);
    }

    function withdraw(uint256 amount) external {
        _burn(msg.sender, amount);
        payable(msg.sender).transfer(amount);
    }
}

contract FlashLender is IERC3156FlashLender {
    function maxFlashLoan(address token) public view returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }

    function flashFee(address token, uint256 amount) public view returns (uint256) {
        return amount / 1000;
    }

    function flashLoan(IERC3156FlashBorrower receiver, address token, uint256 amount, bytes calldata data) external returns (bool) {
        require(amount <= maxFlashLoan(token), "FlashLender: Insufficient funds");
        uint256 fee = flashFee(token, amount);
        IERC20(token).approve(address(receiver), amount);
        require(
            receiver.onFlashLoan(msg.sender, token, amount, fee, data) == keccak256("ERC3156FlashBorrower.onFlashLoan"),
            "FlashMinter: Do not support IERC3156FlashBorrower"
        );
        uint256 _allowance = IERC20(token).allowance(address(receiver), address(this));
        require(_allowance >= (amount + fee), "FlashMinter: Repay not approved");
        IERC20(token).transferFrom(address(receiver), address(this), amount + fee);
        return true;
    }
}
