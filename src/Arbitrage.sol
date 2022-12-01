// SPDX-License-Identifier: AGPL-3.0-only
pragma solidity ^0.8.13;

import "../lib/solmate/src/auth/Owned.sol";
import "../lib/solmate/src/utils/SafeTransferLib.sol";
import "../lib/solmate/src/utils/ReentrancyGuard.sol";

error Unauthorized();
error UncleBlock();
error SelfCall();
error SufficientIncome();
error CoinbaseCall();

interface IERC20 {
    function balanceOf(address account) external view returns (uint256);
    function transfer(address recipient, uint256 amount) external returns (bool);
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

contract Arbitrage is Owned, ReentrancyGuard, IERC3156FlashBorrower {
    using SafeTransferLib for IERC20;

    constructor() Owned(msg.sender) {}

    modifier onlyOwner() override {
        if (msg.sender != owner) revert Unauthorized();

        _;
    }

    function onFlashLoan(
        address initiator,
        address token,
        uint256 amount,
        uint256 fee,
        bytes calldata data
    ) external nonReentrant returns (bytes32) {
        if (tx.origin != owner) revert Unauthorized();
        uint256 balance_before = address(this).balance;

        (bytes32 _parentHash, uint256 _coinbaseFee, bytes[] memory _multicallData) = abi.decode(data, (bytes32, uint256, bytes[]));
        if ((_parentHash != bytes32("")) && (blockhash(block.number - 1) != _parentHash)) revert UncleBlock();
        // multicall
        for (uint256 i = 0; i < _multicallData.length; i++) {
            (address _to, uint256 _value, bytes memory _data) = abi.decode(_multicallData[i], (address, uint256, bytes));
            if (_to == address(this)) revert SelfCall();
            _to.call{ value: _value }(_data);
        }

        // to show reentrant attack, if only consider EOA coinbase, use `transfer` instead
        (bool _success, ) = block.coinbase.call{ value: _coinbaseFee, gas: 100000 }("");
        if(!_success) revert CoinbaseCall();

        uint256 balance_after = address(this).balance;
        // onFlashLoan only checks the ETH balance, not ERC20.
        // Recommend switch to ETH after each arbitrage.
        if (balance_after <= balance_before) revert SufficientIncome();

        return keccak256("ERC3156FlashBorrower.onFlashLoan");
    }

    function withdraw() external onlyOwner {
        payable(msg.sender).transfer(address(this).balance);
    }

    function recoverERC20(address token) external onlyOwner {
        IERC20(token).transfer(msg.sender, IERC20(token).balanceOf(address(this)));
    }

    fallback() external payable {}
}
