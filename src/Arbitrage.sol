// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "solmate/auth/Owned.sol";
import "solmate/utils/SafeTransferLib.sol";

error Unauthorized();
error UncleBlock();
error SelfCall();
error SufficientIncome();
error FlashLenderCall();
error CoinbaseCall();

interface IERC20 {
    function balanceOf(address account) external view returns (uint256);
    function transfer(address recipient, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function allowance(address owner, address spender) external view returns (uint256);
    function approve(address spender, uint256 amount) external returns (bool);
}

interface IERC3156FlashBorrower {
    /**
     * @dev Receive a flash loan.
     * @param initiator The initiator of the loan.
     * @param token The loan currency.
     * @param amount The amount of tokens lent.
     * @param fee The additional amount of tokens to repay.
     * @param data Arbitrary data structure, intended to contain user-defined parameters.
     * @return The keccak256 hash of "ERC3156FlashBorrower.onFlashLoan"
     */
    function onFlashLoan(
        address initiator,
        address token,
        uint256 amount,
        uint256 fee,
        bytes calldata data
    ) external returns (bytes32);
}

interface IERC3156FlashLender {

    /**
     * @dev The amount of currency available to be lent.
     * @param token The loan currency.
     * @return The amount of `token` that can be borrowed.
     */
    function maxFlashLoan(
        address token
    ) external view returns (uint256);

    /**
     * @dev The fee to be charged for a given loan.
     * @param token The loan currency.
     * @param amount The amount of tokens lent.
     * @return The amount of `token` to be charged for the loan, on top of the returned principal.
     */
    function flashFee(
        address token,
        uint256 amount
    ) external view returns (uint256);

    /**
     * @dev Initiate a flash loan.
     * @param receiver The receiver of the tokens in the loan, and the receiver of the callback.
     * @param token The loan currency.
     * @param amount The amount of tokens lent.
     * @param data Arbitrary data structure, intended to contain user-defined parameters.
     */
    function flashLoan(
        IERC3156FlashBorrower receiver,
        address token,
        uint256 amount,
        bytes calldata data
    ) external returns (bool);
}

contract Arbitrage is Owned, IERC3156FlashBorrower {
    using SafeTransferLib for IERC20;

    address flashLender;

    constructor() Owned(msg.sender) {}

    modifier onlyOwner() override {
        if (msg.sender != owner) revert Unauthorized();

        _;
    }

    function setFlashLender(address _lender) public onlyOwner {
        flashLender = _lender;
    }

    function _arbitrage(bytes calldata data) internal {
        uint256 balance_before = address(this).balance;

        // parse data
        (bytes32 _parentHash, uint256 _coinbaseFee, bytes[] memory _multicallData) = abi.decode(data, (bytes32, uint256, bytes[]));
        if ((_parentHash != bytes32("")) && (blockhash(block.number - 1) != _parentHash)) revert UncleBlock();

        // multicall
        for (uint256 i = 0; i < _multicallData.length; i++) {
            (address _to, uint256 _value, bytes memory _data) = abi.decode(_multicallData[i], (address, uint256, bytes));
            _to.call{ value: _value }(_data);
        }

        block.coinbase.transfer(_coinbaseFee);
        uint256 balance_after = address(this).balance;
        // onFlashLoan only checks the ETH balance, not ERC20.
        // Recommend switch to ETH after each arbitrage.
        if (balance_after <= balance_before) revert SufficientIncome();
    }

    function run(bytes calldata data) external onlyOwner {
        _arbitrage(data);
    }

    function onFlashLoan(
        address initiator,
        address token,
        uint256 amount,
        uint256 fee,
        bytes calldata data
    ) public returns (bytes32) {
        if (msg.sender != flashLender) revert FlashLenderCall();
        if (initiator != owner) revert Unauthorized();

        uint256 _allowance = IERC20(token).allowance(address(this), msg.sender);
        IERC20(token).approve(msg.sender, _allowance + amount + fee);

        _arbitrage(data);

        return keccak256("ERC3156FlashBorrower.onFlashLoan");
    }

    function withdraw() external onlyOwner {
        payable(msg.sender).transfer(address(this).balance);
    }

    function recoverERC20(address token) external onlyOwner {
        IERC20(token).transfer(msg.sender, IERC20(token).balanceOf(address(this)));
    }

    receive() external payable {}
}
